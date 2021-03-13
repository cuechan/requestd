use crate::NodeDb;
use crate::CONFIG;
use http;
use log::{error, warn, info, trace};
use serde_json as json;
use socket2;
use std;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::{self, exit, Child, Command, Stdio};
use tiny_http::{self, Request, Response, Server, StatusCode};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum HookError {
	NoHook,
	ExitError(i32),
	IOError(String),
}

impl From<json::Error> for HookError {
	fn from(e: json::Error) -> Self {
		Self::IOError(e.to_string())
	}
}

impl From<std::io::Error> for HookError {
	fn from(e: std::io::Error) -> Self {
		HookError::IOError(e.to_string())
	}
}

pub fn process_request(hookname: String, data: json::Value) -> Result<Vec<u8>, HookError> {
	trace!("try running hook: {:#?}", hookname);
	let hook = match CONFIG.web_endpoints.iter().find(|x| x.path == hookname) {
		Some(e) => e,
		None => {
			warn!("no endpoint configured");
			return Err(HookError::NoHook);
		}
	};

	let res = hook_runner(&hook.exec, json::to_string_pretty(&data)?)?;
	Ok(res)
}

pub fn hook_runner(path: &String, input: String) -> Result<Vec<u8>, HookError> {
	let mut cmd: Child = Command::new(&path)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.spawn()?;

	let mut std_writer = BufWriter::new(cmd.stdin.take().unwrap());
	let mut std_reader = BufReader::new(cmd.stdout.take().unwrap());
	// i don't like this solution but i haven't found a better one
	std::thread::spawn(move || {
		#[allow(unused_must_use)]
		match std_writer.write_all(input.as_bytes()) {
			Err(e) => warn!("hook don't want my input :("),
			Ok(r) => (),
		}
	});

	// read the output from stdout to out
	let mut out: Vec<u8> = Vec::new();
	std_reader.read_to_end(&mut out).unwrap();

	let exit_status = cmd.wait()?;
	if !exit_status.success() {
		return Err(HookError::ExitError(exit_status.code().unwrap()))
	}

	info!("response payload is {} bytes", out.len());
	Ok(out)
}

// pub fn start_webendpoint(nodedb: NodeDb) {
// 	let server: Server = match tiny_http::Server::http(&CONFIG.web_listen) {
// 		Err(e) => {
// 			error!("can't start http server for web_hooks: {}", e);
// 			exit(1);
// 		}
// 		Ok(r) => r,
// 	};

// 	for req in server.incoming_requests() {
// 		info!("a new request for {}", req.url());
// 		process_request(req, nodedb.clone());
// 	}
// }
