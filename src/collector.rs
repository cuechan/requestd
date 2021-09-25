#![allow(unused_must_use)]

use crate::multicast::RequesterService;
use crate::NodeId;
use crate::CONFIG;
use crate::NodeResponse;
use crossbeam;
use crossbeam::channel::{self, Receiver, Sender};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use serde_json as json;
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::io;
use std::time::Instant;


#[derive(Clone)]
pub struct Collector {
	received_counter: usize,
	requester: RequesterService,
	buffer: ResponseBuffer,
	event_senders: Vec<Sender<NodeResponse>>,
}



impl Collector {
	/// Starts a collector thread that also checks the database for offline nodes
	pub fn new(requester: RequesterService) -> Self {
		Self {
			requester,
			received_counter: 0,
			buffer: ResponseBuffer::new(CONFIG.requestd.clone().retention),
			event_senders: vec![]
		}
	}

	// pub fn get_event_emitter(&self) -> Receiver<Event> {
	// 	self.events_channel.1.clone()
	// }

	pub fn start_collector(&self) {
		info!("start collector",);
	}


	pub fn receive(&mut self, response: NodeResponse) {
		// *self.received_counter.lock().unwrap() += 1;
		self.notify_receivers(response.clone());
		self.buffer.receive(response.clone());
	}

	fn notify_receivers(&self, msg: NodeResponse) {
		// send data to all subscribed listeners
		for sender in &self.event_senders {
			sender.try_send(msg.clone());
		}
	}

	pub fn all_responses(&mut self) -> Vec<NodeResponse> {
		let all_responses = self.buffer.get_all_responses();
		all_responses
	}

	pub fn get_num_received(&self) -> usize {
		self.received_counter
	}

	pub fn get_events_receiver(&mut self) -> Receiver<NodeResponse> {
		let (tx, rx) = channel::unbounded();
		self.event_senders.push(tx);
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



#[derive(Clone)]
pub struct ResponseBuffer {
	responses: HashMap<NodeId, NodeResponse>,
	// receiver: Receiver<NodeResponse>,
	max_age: u64,
	last_clean: Instant,
}

impl ResponseBuffer {
	fn new(max_age: u64) -> Self {
		Self {
			responses: HashMap::new(),
			// receiver: events,
			max_age,
			last_clean: Instant::now(),
		}
	}

	fn receive(&mut self, response: NodeResponse) {
		self.responses.insert(response.nodeid.clone(), response.clone());
	}

	fn get_all_responses(&mut self) -> Vec<NodeResponse> {
		// cleaning takes up to 100ms (in debug mode)
		// inly do in once in a while
		if self.last_clean.elapsed().as_secs() > crate::RESPONSE_CLEANING_MAX  {
			self.clean_responses();
		}

		let all_responses = self.responses.values().map(|n| n.clone()).collect();
		all_responses
	}

	/// searches the database for offline nodes and trigger events
	fn clean_responses(&mut self) {
		let t = Instant::now();
		debug!("checking {} entries for dead responses", self.responses.len());

		let mut i = 0;
		for (id, response) in self.responses.clone().iter() {
			// did he dieded?
			if response.age() > self.max_age {
				// trace!("purging node: {}", id);
				self.responses.remove(id);
				i += 1;
			}
		}

		self.last_clean = Instant::now();
		debug!("removed {} entries", i);
		debug!("cleanup took: {}ms ", t.elapsed().as_millis());
	}

	// let collector_c = collector.clone();
	// thread::spawn(move || {
	// 	loop {
	// 		collector_c.lock().unwrap().evaluate_database();
	// 		thread::sleep(Duration::from_secs(CONFIG.requestd.clean_interval));
	// 	}
	// });
}
