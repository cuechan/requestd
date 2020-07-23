#![allow(unused_must_use)]

pub mod nodedb;

use crate::config;
use crate::NodeResponse;
use crate::CONFIG;
use crossbeam_channel as crossbeam;
use crossbeam_channel::{Receiver, Sender};
use jq_rs as jq;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use nodedb::Node;
use nodedb::NodeDb;
use nodedb::NodeStatus;
use serde_json as json;
use serde_json::Value;
use std::fmt::{self, Display};
use std::io;
use std::process::{self, Command};
use std::thread;
use std::time::Duration;

pub struct Collector {
	db: nodedb::NodeDb,
	er: EventRunner,
}

#[derive(Copy, Clone, Debug)]
pub enum Event {
	NewNode,
	NodeOffline,
	NodeOnlineAfterOffline,
	NodeUpdate,
	RemoveNode,
}

impl Display for Event {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl Event {
	pub fn get_configured_hooks(self) -> Vec<config::Event> {
		match self {
			Self::NewNode => CONFIG.events.new_node.clone(),
			Self::NodeOffline => CONFIG.events.node_offline.clone(),
			_ => vec![],
		}
	}
}

impl Collector {
	/// Starts a collector thread that also checks the database for offline nodes
	pub fn new(db: NodeDb) -> Self {
		let er = EventRunner::new();

		let mut db_copy = db.clone();
		let mut er_copy = er.clone();
		thread::spawn(move || {
			// wait an initial period to prevent that all nodes appear to be offline
			thread::sleep(Duration::from_secs(CONFIG.database.offline_after as u64));

			loop {
				trace!("checking for offline nodes");
				// get all nodes, that are marked as online and check
				// if we actually got a message in the last n seconds
				// or if we have to assume that it went offline
				for n in db_copy.get_all_nodes().into_iter() {
					if !n.is_online() && n.status != NodeStatus::Down {
						trace!("offline node found: {}", n.nodeid);
						// first, mark node as offline
						db_copy.set_status(&n.nodeid, NodeStatus::Down);
						// then trigger the event
						er_copy.push_event(Event::NodeOffline, n.clone());
					}

					// did he dieded?
					if n.is_dead() {
						trace!("purging node: {}", n.nodeid);
						// remove node from db
						db_copy.delete_node(&n.nodeid);
						// then trigger the event
						er_copy.push_event(Event::RemoveNode, n.clone());
					}
				}

				thread::sleep(Duration::from_secs(CONFIG.database.evaluate_every as u64))
			}
		});

		Self { db, er }
	}

	pub fn receive(&mut self, nodedata: NodeResponse) {
		trace!("Node: {:#?}", nodedata.nodeid);

		let node = if !self.db.is_known(&nodedata.nodeid) {
			trace!("Node is not known");
			self.db.insert_node(&nodedata).unwrap();
			let node = self.db.get_node(&nodedata.nodeid).unwrap();

			self.er.push_event(Event::NewNode, node.clone());
			node
		} else {
			trace!("We know this node");
			self.db.get_node(&nodedata.nodeid).unwrap()
		};

		if node.status == NodeStatus::Down {
			self.er.push_event(Event::NodeOnlineAfterOffline, node.clone());
		}

		self.db.insert_node(&nodedata).unwrap();
		self.er.push_event(Event::NodeUpdate, node);
	}
}

#[derive(Clone)]
pub struct EventRunner {
	threads: Sender<(config::Event, Node)>,
}

impl EventRunner {
	fn new() -> Self {
		let (sender, receiver) = crossbeam::unbounded();

		for i in 0..CONFIG.concurrent_hooks - 1 {
			let own_receiver = receiver.clone();

			thread::Builder::new()
				.name(format!("hook_runner_{}", i))
				.spawn(move || hook_worker(own_receiver))
				.unwrap();
		}

		Self { threads: sender }
	}

	pub fn push_event(&mut self, e: Event, n: Node) {
		trace!("Event Triggered: {} for {}", e, n.nodeid);

		match e {
			Event::NodeOffline => {
				// info!("node is offline {}", n.nodeid);

				for hook in &CONFIG.events.node_offline {
					self.threads.send((hook.clone(), n.clone())).unwrap();
				}
			}
			Event::NewNode => {
				// info!("a new node {}", n.nodeid);
				for hook in &CONFIG.events.new_node {
					self.threads.send((hook.clone(), n.clone())).unwrap();
				}
			}
			Event::NodeUpdate => {
				for hook in &CONFIG.events.node_update {
					self.threads.send((hook.clone(), n.clone())).unwrap();
				}
			}
			Event::NodeOnlineAfterOffline => {
				for hook in &CONFIG.events.online_after_offline {
					self.threads.send((hook.clone(), n.clone())).unwrap();
				}
			}
			_ => warn!("event not supported yet: {}", e),
		}

		debug!("queue size {}", self.threads.len());

		// thread::sleep(Duration::from_millis(500))
	}
}

fn hook_worker(receiver: Receiver<(config::Event, Node)>) {
	// 	let mut db = triggerdb::TriggerDb::new(Arc::new(Mutex::new(
	// 		sqlite::Connection::open(&CONFIG.database.dbfile).unwrap()
	// 	)));

	for (event, n) in receiver {
		trace!("running hook: {}", event.exec);
		#[allow(unused_must_use)]
		event_trigger(event.clone(), n).map_err(|e| {
			error!("running hook '{}' failed: {}", event.exec, e);
		});
	}
}

pub fn event_trigger(event: config::Event, n: Node) -> Result<(), EventError> {
	let vars = event.vars.iter().map(|(var, q)| {
		let val = jq::compile(q)
			.unwrap()
			.run(&json::to_string(&n.last_response).unwrap())
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
	json::to_writer(stdin, &n.last_response);

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
