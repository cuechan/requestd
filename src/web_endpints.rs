use crate::NodeDb;
use crate::CONFIG;
use http;
use log::{error, info, warn};
use serde_json as json;
use socket2;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::{self, Child, Command, Stdio};
use tiny_http::{self, Request, Response, Server, StatusCode};

const ADDRESS: &str = "[::]:21001";

fn process_request(req: Request, db: NodeDb) {
	let hook = match CONFIG.web_endpoints.iter().find(|x| x.path == req.url()) {
		Some(e) => e,
		None => {
			error!("no endpoint configured");
			req.respond(Response::new_empty(StatusCode::from(404))).unwrap();
			return;
		}
	};

	let mut cmd: Child = match Command::new(&hook.exec)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.spawn()
	{
		Ok(c) => c,
		Err(e) => {
			error!("hook script not found: {:#?}", hook.exec);
			req.respond(Response::new_empty(StatusCode::from(404))).unwrap();
			return;
		}
	};

	let allnodes = db.get_all_nodes();
	let writer = BufWriter::new(cmd.stdin.unwrap());
	// i don't like this solution but i haven't found a better one
	std::thread::spawn(move || {
		#[allow(unused_must_use)]
		match json::to_writer(writer, &allnodes) {
			Err(e) => warn!("hook don't want my input :("),
			Ok(r) => (),
		}
	});

	// let reader = BufReader::new(stdout);
	let mut out: Vec<u8> = Vec::new();
	let len = cmd.stdout.unwrap().read_to_end(&mut out).unwrap();
	info!("response payload is {} bytes", out.len());

	let response = Response::new(StatusCode::from(200), vec![], out.as_slice(), None, None);
	req.respond(response).unwrap();
	// cmd.wait().unwrap();
}

pub fn start_webendpoint(nodedb: NodeDb) {
	let server: Server = tiny_http::Server::http(ADDRESS).unwrap();

	for req in server.incoming_requests() {
		info!("a new request for {}", req.url());
		process_request(req, nodedb.clone());
	}
}
