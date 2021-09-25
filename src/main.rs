#![feature(proc_macro_hygiene, decl_macro)]

pub mod collector;
pub mod config;
pub mod multicast;
pub mod web;
pub mod mqtt;

use chrono::{DateTime, Utc};
use clap;
use collector::Collector;
use config::Config;
use lazy_static::lazy_static;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use pretty_env_logger;
use serde_json as json;
use serde_yaml as yaml;
use serde::{Serialize, Deserialize};
use std::net::IpAddr;
use std::process;
use std::thread;
use std::time::Duration;
use std::sync::{Mutex, Arc};
use config::ConfigLoadingError;

pub const DEFAULT_CONF_FILES: &[&str] = &["requestd.yml", "/etc/requestd.yml"];
pub const RESPONSE_CLEANING_MAX: u64 = 10;

pub type NodeData = json::Value;
pub type Timestamp = DateTime<Utc>;
pub type NodeId = String;
pub type Mac = String;


lazy_static! {
	pub static ref CONFIG: Config = {
		config::Config::load_config(DEFAULT_CONF_FILES).map_err(|e| {
			match e {
				ConfigLoadingError::NoConfigFound => error!("no config found. First file in some of these locations will be loaded: {}", DEFAULT_CONF_FILES.join(", ")),
				ConfigLoadingError::Io(e) => error!("error while loading config file: {}", e),
				ConfigLoadingError::Yaml(e) => error!("error while parsing config: {}", e),
			}

			error!("generate a default config with '{} config -d'", env!("CARGO_BIN_NAME"));
			process::exit(1);
		}).unwrap()
	};
}

fn main() {
	let args = clap::App::new(env!("CARGO_BIN_NAME"))
		.author(env!("CARGO_PKG_AUTHORS"))
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.version(env!("CARGO_PKG_VERSION"))
		.subcommand(clap::SubCommand::with_name("config")
			.about("print config")
			.arg(clap::Arg::with_name("default")
				.long("default")
				.short("d")
				.help("print the default configuration")
			)
		).get_matches();

	pretty_env_logger::init();

	match args.subcommand() {
		("config", Some(args)) => cmd_config(args),
		_ => start_collecting()
	}
}



fn cmd_config(args: &clap::ArgMatches) {
	if args.is_present("default") {
		println!("{}", yaml::to_string(&Config::default()).unwrap());
	}
	else {
		println!("{}", yaml::to_string(&CONFIG.clone()).unwrap());
	}
}






// TODO: this needs a bit more/clearer structure
fn start_collecting() {
	let requester = multicast::RequesterService::new(&CONFIG.requestd.interface);
	let receiver = requester.get_receiver();


	let collector = Arc::new(Mutex::new(Collector::new(requester.clone())));
	collector.lock().unwrap().start_collector();


	if CONFIG.web.is_some() {
		let collector_c = collector.clone();
		let web = web::Web::new(collector_c);
		std::thread::spawn(move || {
			web.start();
		});
	}
	if CONFIG.mqtt.is_some() {
		let collector_c = collector.clone();
		let mqtt = mqtt::Mqtt::new(collector_c);
		std::thread::spawn(move || {
			mqtt.start();
		});
	}


	debug!("starting requester");
	thread::spawn(move || loop {
		debug!("requesting new data");
		requester.request(&CONFIG.requestd.multicast_address, &CONFIG.requestd.categories);
		thread::sleep(Duration::from_secs(CONFIG.requestd.interval));
	});


	trace!("start processing responses");
	for node_response in &receiver {
		// do some checks
		if !node_response.response.is_object() {
			warn!("a node at {} send invalid data", node_response.remote.to_string());
			continue;
		}

		let nodeid = if let Some(nodeid) = get_nodeid_from_response_data(&node_response.response) {
			nodeid
		} else {
			warn!("a node at {} has no nodeid", node_response.remote.to_string());
			continue;
		};

		let node_res = NodeResponse {
			nodeid: nodeid.to_string(),
			remote: node_response.remote.ip(),
			timestamp: node_response.timestamp,
			data: node_response.response,
		};

		collector.lock().unwrap().receive(node_res);
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeResponse {
	nodeid: NodeId,
	remote: IpAddr,
	timestamp: Timestamp,
	data: NodeData,
}

impl NodeResponse {
	fn age(&self) -> u64 {
		(Utc::now() - self.timestamp).num_seconds() as u64
	}
}



fn get_nodeid_from_response_data(data: &json::Value) -> Option<NodeId> {
	data.as_object()
		.and_then(|n| Some(n.iter()))
		.and_then(|mut n| n.nth(0))
		.and_then(|n| Some(n.1))
		.and_then(|n| n.get("node_id"))
		.and_then(|n| n.as_str())
		.and_then(|n| Some(n.to_string())) as Option<NodeId>
}


#[derive(Debug)]
pub enum Error {}

// impl From<influxdb::Error> for Error {
// 	fn from(e: influxdb::Error) -> Error {
// 		Error::Influx(e)
// 	}
// }


pub trait Endpoint {
	/// create and initialize new endpoint
	fn new(c: Arc<Mutex<Collector>>) -> Self;

	/// start the Endpoint
	/// This method must never return. It will be started in a seperate thread
	fn start(self) -> !;
}
