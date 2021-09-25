use crate::Collector;
use crate::Endpoint;
use crate::NodeResponse;
use crate::CONFIG;
use crossbeam::channel as crossbeam;
use log::{trace};
use zmq::{self, SocketType};
use serde_json as json;
use std::sync::{Arc, Mutex};

const ZMQ_TOPIC: &str = "requestd";

pub struct Zmq {
	zsocket: zmq::Socket,
	events_receiver: crossbeam::Receiver<NodeResponse>,
}

// impl Zmq {
// }

impl Endpoint for Zmq {
	fn new(c: Arc<Mutex<Collector>>) -> Self {
		let ctx = zmq::Context::new();
		let zsocket = ctx.socket(SocketType::PUB).unwrap();
		zsocket.bind(&CONFIG.zmq.clone().unwrap().bind_to).unwrap();

		Self {
			zsocket: zsocket,
			events_receiver: c.lock().unwrap().get_events_receiver(),
		}
	}

	fn start(self) -> ! {
		for event in &self.events_receiver {
			trace!("sending zmq message");
			self.zsocket.send(ZMQ_TOPIC, zmq::SNDMORE).unwrap();
			self.zsocket.send(json::to_vec(&event).unwrap(), 0).unwrap();
		}

		panic!("event loop stopped");
	}
}
