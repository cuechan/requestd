#![allow(unused_must_use)]

use crate::multicast::RequesterService;
use crate::NodeId;
use crate::NodeResponse;
use crate::CONFIG;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use serde_json as json;
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::io;
use crossbeam::channel as crossbeam;


#[derive(Clone)]
pub struct Collector {
	received_counter: usize,
	requester: RequesterService,
	responses: HashMap<NodeId, NodeResponse>,
	events_senders: Vec<crossbeam::Sender<NodeResponse>>,
}



impl Collector {
	/// Starts a collector thread that also checks the database for offline nodes
	pub fn new(requester: RequesterService) -> Self {
		Self {
			requester,
			received_counter: 0,
			responses: HashMap::new(),
			events_senders: vec![]
		}
	}

	// pub fn get_event_emitter(&self) -> Receiver<Event> {
	// 	self.events_channel.1.clone()
	// }

	pub fn start_collector(&self) {
		info!("start collector",);
	}


	/// searches the database for offline nodes and trigger events
	pub fn evaluate_database(&mut self) {
		debug!("checking for invalid responses");

		// get all nodes, that are marked as online and check
		// if we actually got a message in the last n seconds
		// or if we have to assume that it went offline
		for (id, response) in self.responses.clone().iter() {
			// did he dieded?
			if response.age() > CONFIG.requestd.retention {
				trace!("purging node: {}", id);
				// remove node from db
				self.responses.remove(id);
			}
		}
	}

	pub fn receive(&mut self, response: NodeResponse) {
		// *self.received_counter.lock().unwrap() += 1;
		self.responses.insert(response.nodeid.clone(), response.clone());
		self.notify_receivers(response);
	}

	fn notify_receivers(&self, msg: NodeResponse) {
		// send data to all subscribed listeners
		for sender in &self.events_senders {
			sender.try_send(msg.clone());
		}
	}

	pub fn all_responses(&self) -> Vec<NodeResponse> {
		let all_responses = self.responses.values().map(|n| n.clone()).collect();
		all_responses
	}

	pub fn get_num_received(&self) -> usize {
		self.received_counter
	}

	pub fn get_events_receiver(&mut self) -> crossbeam::Receiver<NodeResponse> {
		let (tx, rx) = crossbeam::unbounded();
		self.events_senders.push(tx);
		rx
	}
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
