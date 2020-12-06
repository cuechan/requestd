use crate::NodeDb;
use crate::CONFIG;
use http;
use log::{error, info, warn};
use serde_json as json;
use socket2;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::{self, exit, Child, Command, Stdio};
use tiny_http::{self, Request, Response, Server, StatusCode};

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
			error!("starting hook script failed: {}", e);
			req.respond(Response::new_empty(StatusCode::from(404))).unwrap();
			return;
		}
	};

	let allnodes = db.get_all_nodes();
	let std_writer = BufWriter::new(cmd.stdin.take().unwrap());
	let mut std_reader = BufReader::new(cmd.stdout.take().unwrap());

	// i don't like this solution but i haven't found a better one
	std::thread::spawn(move || {
		#[allow(unused_must_use)]
		match json::to_writer(std_writer, &allnodes) {
			Err(e) => warn!("hook don't want my input :("),
			Ok(r) => (),
		}
	});

	// read the output from stdout to out
	let mut out: Vec<u8> = Vec::new();
	std_reader.read_to_end(&mut out).unwrap();
	info!("response payload is {} bytes", out.len());

	// create a response
	let response = Response::new(StatusCode::from(200), vec![], out.as_slice(), None, None);
	if let Err(e) = req.respond(response) {
		warn!("could not respond to web requestd: {}", e);
	}

	// wait for the process to exit
	cmd.wait().unwrap();
}

pub fn start_webendpoint(nodedb: NodeDb) {
	let server: Server = match tiny_http::Server::http(&CONFIG.web_listen) {
		Err(e) => {
			error!("can't start http server for web_hooks: {}", e);
			exit(1);
		}
		Ok(r) => r,
	};

	for req in server.incoming_requests() {
		info!("a new request for {}", req.url());
		process_request(req, nodedb.clone());
	}
}
