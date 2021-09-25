#[allow(unused_imports)]
use crate::collector::{self, Collector, ResponseBuffer};
use crate::CONFIG;
use crate::Endpoint;
use crate::NodeResponse;
use crossbeam::channel::Receiver;
use log::{error, warn};
use std;
use std::net::SocketAddr;
use std::process;
use std::sync::{Arc, Mutex};
use tiny_http::{Server, Response, Request, Header};
use serde_json as json;
use std::thread;


const DATETIME_FORMAT: &str = "%F %T";

pub struct Web {
	collector: Arc<Mutex<Collector>>,
	server: Server,
}


fn handle_index(req: Request) {
	let mut res = Response::from_string(include_str!("index.html"));
	res.add_header(Header::from_bytes("Content-Type", "text/html").unwrap());

	req.respond(res).unwrap();
}


fn handle_responses(req: Request, all_nodes: Vec<NodeResponse>) {
	let mut res = Response::from_data(json::to_vec(&all_nodes).unwrap());
	res.add_header(Header::from_bytes("Content-Type", "application/json").unwrap());

	req.respond(res).unwrap();
}

impl Endpoint for Web {
	fn new(c: Arc<Mutex<Collector>>) -> Self {
		let server = Server::http(CONFIG.web.clone().unwrap().listen).unwrap();

		Self {
			collector: c,
			server: server,
		}
	}

	fn start(mut self) -> ! {
		for req in self.server.incoming_requests() {
			match req.url() {
				"/responses" => {
					let responses = self.collector.lock().unwrap().all_responses();
					handle_responses(req, responses);
				}
				_ => handle_index(req),
			}
		};

		panic!("http endpoint loop returned. (this should not happen")
	}
}
