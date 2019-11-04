pub mod collector;
pub mod config;
pub mod model;
pub mod multicast;
pub mod output;

use chrono::{DateTime, Utc};
use clap;
use collector::Collector;
use influx_db_client as influxdb;
use log::{debug, error, info, trace, warn};
use multicast::if_to_index;
use multicast::RecordType;
use multicast::ResponddResponse;
use postgres;
use pretty_env_logger;
use rusqlite as sqlite;
use rusqlite::NO_PARAMS;
use serde_json as json;
use std::fs::File;
use std::io::Write;
use std::process;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::net::SocketAddr;
use zmq;

pub const APPNAME: &str = "ffhl-collector";
pub const TABLE: &str = "nodes";
pub const DATABASE_PATH: &str = "./nodes.db";
pub const ZMQ_ENDPOINT: &str = "tcp://tmp/ffhlack.sock";


fn main() {
	// read config files
	let mut clap = clap_app();
	let matches = clap.clone().get_matches();

	if !matches.is_present("quiet") {
		pretty_env_logger::init();
	}

	let interface;

	if let Some(iface) = matches.value_of("iface") {
		interface = iface.to_owned();
	}


	let ctx = zmq::Context::new();
	let zmqpub = ctx.socket(zmq::SocketType::PUB).unwrap();
	zmqpub.bind(ZMQ_ENDPOINT).unwrap();

	let service = multicast::ResponderService::start(
		matches.value_of("interface").unwrap(),
		15
	);

	let receiver = service.get_receiver();

	thread::spawn(move || {
		loop {
			info!("request new data");
			service.request(RecordType::Statisitcs);
			service.request(RecordType::Nodeinfo);
			service.request(RecordType::Neighbours);

			thread::sleep(Duration::from_secs(30));
		}
	});



	for node_response in receiver {


	}

}



pub struct IpcMessage {
	: type,
}





fn clap_app<'a, 'b>() -> clap::App<'a, 'b> {
	clap::App::new(APPNAME)
		.version("0.0.0")
		.arg(clap::Arg::with_name("iface")
			.short("i")
			.long("iface")
			.help("respondd interface")
			.takes_value(true)
			.validator(|x| if_to_index(&x).map_or(Err("no interface".to_owned()), |_| Ok(()))),
		)
		.arg(
			clap::Arg::with_name("delay")
				.short("d")
				.long("delay")
				.help("delay before fetching data in ms")
				.takes_value(true)
				.default_value("0")
				.validator(|x| match x.parse::<u64>() {
					Ok(_) => Ok(()),
					Err(e) => Err(e.to_string()),
				}),
		)
}
