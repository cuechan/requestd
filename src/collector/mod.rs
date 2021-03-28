#![allow(unused_must_use)]

pub mod nodedb;

use crate::config;
use crate::monitor::metrics::HOOKS_WAITING;
use crate::multicast::RequesterService;
use serde::{Serialize, Deserialize};
use crate::NodeResponse;
use crate::CONFIG;
use crossbeam::channel;
use crossbeam::channel::{Receiver, Sender};
use jq_rs as jq;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use nodedb::Node;
use nodedb::NodeDb;
use nodedb::NodeStatus;
use std::str;
use serde_json as json;
use serde_json::Value;
use std::fmt::{self, Display};
use std::io;
use std::process::{self, Command};
use std::thread;
use std::time::Duration;
use std::sync::atomic::*;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use std::str::FromStr;
use crate::NodeId;
use crate::{DEFAULT_EVENT_HISTORY_LIMIT};

#[derive(Clone)]
pub struct Collector {
	nodedb: nodedb::NodeDb,
	events_channel: (Sender<Event>, Receiver<Event>),
	er: EventRunner,
	received_counter: Arc<Mutex<usize>>,
	requester: RequesterService,
}


#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
/// this is a terrible name
pub enum EventEvent {
	NewNode,
	NodeOffline,
	NodeOnlineAfterOffline,
	NodeUpdate,
	RemoveNode,
}

impl EventEvent {
	pub fn get_configured_hooks(self) -> Vec<config::Event> {
		match self {
			Self::NewNode => CONFIG.events.new_node.clone(),
			Self::NodeOffline => CONFIG.events.node_offline.clone(),
			Self::NodeUpdate => CONFIG.events.node_update.clone(),
			Self::NodeOnlineAfterOffline => CONFIG.events.online_after_offline.clone(),
			_ => unimplemented!(),
		}
	}
}

impl rusqlite::types::FromSql for EventEvent {
	fn column_result(value: rusqlite::types::ValueRef) -> rusqlite::types::FromSqlResult<Self> {
		if let rusqlite::types::ValueRef::Text(e) = value {
			return Ok(str::from_utf8(e).unwrap().parse().unwrap())
		}

		Ok(EventEvent::NewNode)
	}
}

impl FromStr for EventEvent {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"NewNode" => Ok(Self::NewNode),
			"NodeOffline" => Ok(Self::NodeOffline),
			"NodeOnlineAfterOffline" => Ok(Self::NodeOnlineAfterOffline),
			"NodeUpdate" => Ok(Self::NodeUpdate),
			"RemoveNode" => Ok(Self::RemoveNode),
			_ => Err("can't parse event".to_string()),
		}
	}
}


impl Display for EventEvent {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}


#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct Event {
	event: EventEvent,
	nodeid: NodeId,
	trigger: String,
	timestamp: DateTime<Utc>,
}

impl Event {
	fn new(eevent: EventEvent, node: &NodeId) -> Self {
		Self {
			event: eevent,
			nodeid: node.clone(),
			trigger: String::new(),
			timestamp: Utc::now(),
		}
	}
}

impl Display for Event {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}({})", self.event, self.nodeid)
	}
}

impl Collector {
	/// Starts a collector thread that also checks the database for offline nodes
	pub fn new(nodedb: NodeDb, requester: RequesterService) -> Self {
		let er = EventRunner::new();

		Self {
			nodedb,
			events_channel: channel::unbounded(),
			requester,
			er,
			received_counter: Arc::new(Mutex::new(0)),
		}
	}

	pub fn get_event_emitter(&self) -> Receiver<Event> {
		self.events_channel.1.clone()
	}

	pub fn start_collector(&self) {
		info!("start collector",);
		let mut self_ = self.clone();

		thread::spawn(move || {
			// wait an initial period to prevent that all nodes appear to be offline
			// thread::sleep(Duration::from_secs(CONFIG.database.offline_after as u64));

			loop {
				self_.evaluate_database();
				thread::sleep(Duration::from_secs(CONFIG.database.evaluate_every as u64))
			}
		});
	}

	fn emit_event<T: Serialize>(&mut self, event: Event, payload: &T) {
		self.nodedb.insert_event(&event);
		self.er.push_event((event.clone(), json::to_value(payload).unwrap()));

		self.events_channel.0.send(event);
	}

	pub fn get_event_history(&self) -> Vec<Event> {
		self.nodedb.get_all_events(DEFAULT_EVENT_HISTORY_LIMIT).clone()
	}

	/// searches the database for offline nodes and trigger events
	fn evaluate_database(&mut self) {
		debug!("checking for offline nodes");
		let mut db_copy = self.nodedb.clone();
		let mut er_copy = self.er.clone();

		// get all nodes, that are marked as online and check
		// if we actually got a message in the last n seconds
		// or if we have to assume that it went offline
		for n in db_copy.get_all_nodes().into_iter() {
			// did he dieded?
			if n.is_dead() {
				trace!("purging node: {}", n.nodeid);
				// remove node from db
				db_copy.delete_node(&n.nodeid);
				// then trigger the event

				self.emit_event(
					Event::new(EventEvent::RemoveNode, &n.nodeid),
					&n
				);

				continue;
			}

			if n.is_offline() && n.status == NodeStatus::Up {
				info!(
					"offline node found: {}, last seen: {}s ago ",
					n.nodeid,
					n.since_last_seen().num_seconds()
				);
				// first, mark node as offline
				db_copy.set_status(&n.nodeid, NodeStatus::Down);
				// then trigger the event
				self.emit_event(
					Event::new(EventEvent::NodeOffline, &n.nodeid),
					&n
				);

				continue;
			}

			// check if the node needs some attention
			if n.since_last_seen().num_seconds() as f64 > (2.0 * CONFIG.respondd.interval as f64)
				&& n.is_online()
			{
				trace!(
					"{:#?}({}) hasn't responded for {}s",
					n.nodeid,
					n.status,
					n.since_last_seen().num_seconds()
				);
				self.requester.request(&n.last_address, &CONFIG.respondd.categories);
			}
		}
	}

	pub fn receive(&mut self, nodedata: NodeResponse) {
		// trace!("Node: {:#?}", nodedata.nodeid);
		*self.received_counter.lock().unwrap() += 1;

		let node: Node = if !self.nodedb.is_known(&nodedata.nodeid) {
			self.nodedb.insert_node(&nodedata).unwrap();
			let node = self.nodedb.get_node(&nodedata.nodeid).unwrap();

			self.emit_event(
				Event::new(EventEvent::NewNode, &node.nodeid),
				&node
			);

			node
		} else {
			self.nodedb.get_node(&nodedata.nodeid).unwrap()
		};

		if node.is_offline() {
			self.emit_event(
				Event::new(EventEvent::NodeOnlineAfterOffline, &node.nodeid),
				&node
			);
		}

		self.nodedb.insert_node(&nodedata).unwrap();
		self.emit_event(Event::new(EventEvent::NodeUpdate, &node.nodeid), &node);
	}

	pub fn get_num_received(&self) -> usize {
		*self.received_counter.lock().unwrap()
	}
}

#[derive(Clone)]
pub struct EventRunner {
	threads: Sender<(config::Event, json::Value)>,
}

impl EventRunner {
	fn new() -> Self {
		let (sender, receiver) = channel::unbounded();

		for i in 0..CONFIG.concurrent_hooks - 1 {
			let own_receiver = receiver.clone();

			thread::Builder::new()
				.name(format!("hook_runner_{}", i))
				.spawn(move || hook_worker(own_receiver))
				.unwrap();
		}

		Self { threads: sender }
	}

	pub fn push_event(&mut self, e: (Event, json::Value)) {
		trace!("Event Triggered: {} for {}", e.0, e.0.nodeid);

		for hook in e.0.event.get_configured_hooks() {
			self.threads.send((hook.clone(), e.1.clone())).unwrap();
		}

		HOOKS_WAITING.set(self.threads.len() as i64);

		// thread::sleep(Duration::from_millis(500))
	}
}

fn hook_worker(receiver: Receiver<(config::Event, json::Value)>) {
	for (event, data) in receiver {
		#[allow(unused_must_use)]
		event_trigger(event.clone(), &data).map_err(|e| {
			error!("running hook '{}' failed: {}", event.exec, e);
		});
	}
}

pub fn event_trigger(event: config::Event, data: &json::Value) -> Result<(), EventError> {
	let vars = event.vars.iter().map(|(var, q)| {
		let val = jq::compile(q)
			.unwrap()
			.run(&json::to_string(&data).unwrap())
			.unwrap();

		// workaround because jq_rs does not support the -r flag
		// True is converted to `1` and false to an empty string
		// this is primarily for bash scripts
		let val_nice = match json::from_str(&val).unwrap() {
			Value::String(s) => s,
			Value::Bool(b) if b => String::new(),
			Value::Bool(b) if !b => 1.to_string(),
			_ => val,
		};

		(var, val_nice)
	});

	let mut cmd = Command::new(&event.exec)
		.envs(vars)
		.stdin(process::Stdio::piped())
		.spawn()?;

	let stdin = cmd.stdin.as_mut().expect("can't get stdin");

	#[allow(unused_must_use)]
	json::to_writer(stdin, &data);

	cmd.wait()?;

	Ok(())
}

#[derive(Debug)]
pub enum EventError {
	Json(json::Error),
	Io(io::Error),
}

impl From<io::Error> for EventError {
	fn from(e: io::Error) -> Self {
		Self::Io(e)
	}
}

impl From<json::Error> for EventError {
	fn from(e: json::Error) -> Self {
		Self::Json(e)
	}
}

impl Display for EventError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Io(e) => write!(f, "{}", e),
			Self::Json(e) => write!(f, "{}", e),
		}
	}
}
