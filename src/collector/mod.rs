use crate::config;
use crate::config::Config;
use crate::NodeResponse;
use jq_rs as jq;
#[allow(unused_imports)]
use log::{debug, error, warn, info, trace};
use nodedb::Node;
use serde_json as json;
use serde_json::Value;
use std::io;
use std::process::{self, Command};
use std::rc::Rc;
use crossbeam_channel as crossbeam;
use crossbeam_channel::{Sender, Receiver};
use std::thread;
use std::time::Duration;
use crate::CONFIG;
use std::sync::{Arc, Mutex};
use rusqlite as sqlite;
use triggerdb::Trigger;
use nodedb::NodeStatus;
use std::fmt::{self, Display};
use crate::HOOK_RUNNER;

pub mod nodedb;
pub mod triggerdb;


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
			_ => vec![]
		}
	}
}


impl Collector {
	pub fn new() -> Self {
		let db = nodedb::NodeDb::new();
		let er = EventRunner::new();

		let mut db_copy = db.clone();
		let mut er_copy = er.clone();
		thread::spawn(move || {
			loop {
				trace!("checking for offline nodes");
				for n in db_copy.get_all_nodes().into_iter().filter(|n| (n.is_offline() && n.status != NodeStatus::Down)) {
					trace!("dead node found: {}", n.nodeid);
					er_copy.push_event(Event::NodeOffline, n.clone());

					db_copy.set_status(n.nodeid, NodeStatus::Down);
				}

				thread::sleep(Duration::from_secs(CONFIG.database.evaluate_every as u64))
			}
		});

		Self {db, er}
	}


	pub fn receive(&mut self, nodedata: NodeResponse) {
		trace!("Node: {:#?}", nodedata.nodeid);

		let node = if !self.db.is_known(&nodedata.nodeid) {
			self.db.insert_node(&nodedata).unwrap();
			let node = self.db.get_node(&nodedata.nodeid).unwrap();

			self.er.push_event(Event::NewNode, node.clone());
			node
		}
		else {
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

		for i in 0..HOOK_RUNNER {
			let own_receiver = receiver.clone();

			thread::Builder::new()
				.name(format!("hook_runner_{}", i))
				.spawn(move || hook_worker(own_receiver))
				.unwrap();
		}

		Self {
			threads: sender
		}
	}


	pub fn push_event(&mut self, e: Event, n: Node) {
		debug!("Event Triggered: {} for {}", e, n.nodeid);

		match e {
			Event::NodeOffline => {
				for hook in &CONFIG.events.node_offline {
					self.threads.send((hook.clone(), n.clone())).unwrap();
				}
			},
			Event::NewNode => {
					for hook in &CONFIG.events.new_node {
						self.threads.send((hook.clone(), n.clone())).unwrap();
					}
			},
			Event::NodeUpdate => {
				for hook in &CONFIG.events.new_node {
					self.threads.send((hook.clone(), n.clone())).unwrap();
				}
			}
			_ => (),
		}

		debug!("queue size {}", self.threads.len());

		// thread::sleep(Duration::from_millis(500))
	}
}





fn hook_worker(receiver: Receiver<(config::Event, Node)>) {
// 	let mut db = triggerdb::TriggerDb::new(Arc::new(Mutex::new(
// 		sqlite::Connection::open(&CONFIG.database.dbfile).unwrap()
// 	)));

	for (e, n) in receiver {
		// trace!("recieved event: {:#?}", e);
		event_trigger(e, n).map_err(|e| {
			error!("running hook failed: {}", e);
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
		let val_nice = match json::from_str(&val).unwrap() {
			Value::String(s) => s,
			Value::Bool(b) if b => String::new(),
			Value::Bool(b) if !b => 1.to_string(),
			_=> val,
		};

		(var, val_nice)
	});


	let mut cmd = Command::new(&event.exec)
		.envs(vars)
		.stdin(process::Stdio::piped())
		.spawn()?;

	let stdin = cmd.stdin.as_mut().expect("can't get stdin");

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
