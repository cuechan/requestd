use chrono::{DateTime, Utc};
use clap;
use collector::Collector;
use collector::nodedb;
use config::Config;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use multicast::if_to_index;
use pretty_env_logger;
use rusqlite as sqlite;
use rusqlite::NO_PARAMS;
use serde_json as json;
use serde_yaml as yaml;
use std::fs::File;
use std::net::IpAddr;
use std::process;
use std::process::exit;
use std::io;
use std::thread;
use std::time::Duration;
use nodedb::NodeId;
use lazy_static::lazy_static;
use nodedb::Node;

pub mod collector;
pub mod config;
pub mod model;
pub mod multicast;
pub mod output;

pub const APPNAME: &str = "ffhl-collector";
pub const TABLE: &str = "nodes";
pub const DATABASE_PATH: &str = "./nodes.db";
pub const DEFAULT_CONF_FILE: &str = "./config.yml";
pub const DEFAULT_MIN_ACTIVE: u64 = 1209600;
pub const DEFAULT_OFFLINE_THRESH: u64 = 120;
pub const HOOK_RUNNER: u64 = 4;

pub type NodeData = json::Value;
pub type Timestamp = DateTime<Utc>;


lazy_static!{
	pub static ref ARGS: clap::ArgMatches<'static> = clap_app().get_matches();
	pub static ref CONFIG: Config = {
		config::Config::load_config(&ARGS).map_err(|e| {
			error!("loading config: {}", e);
			exit(1);
		}).unwrap()
	};
}


fn main() {
	pretty_env_logger::init();


	trace!("config: \n{}", yaml::to_string(&*CONFIG).unwrap());

	match ARGS.subcommand() {
		("ls-nodes", m) => {
			cmd_ls_nodes(m.unwrap().clone());
		},
		("collect", _m) => {
			cmd_collect();
		}
		_ => {
			error!("not a valid Command. Try --help");
			process::exit(1);
		}
	}
}




fn cmd_collect() {
	let requester = multicast::ResponderService::start(&CONFIG.respondd.interface, CONFIG.respondd.interval);
	let receiver = requester.get_receiver();

	thread::spawn(move || {
		loop {
			debug!("request new data");
			requester.request(&CONFIG.respondd.categories);

			thread::sleep(Duration::from_secs(CONFIG.respondd.interval));
		}
	});


	let mut cllctr = Collector::new();

	for node_response in &receiver {
		// do some checks
		if !node_response.response.is_object() {
			warn!("a node at {} send invalid data", node_response.remote.to_string());
			continue;
		}

		let nodeid = if let Some(nodeid) = get_nodeid_from_response_data(&node_response.response) {nodeid}
		else {
			warn!("a node at {} has no nodeid", node_response.remote.to_string());
			continue;
		};

		// debug!("Hello, {}", nodeid);

		let noderes = NodeResponse {
			nodeid: nodeid.to_string(),
			remote: node_response.remote.ip(),
			timestamp: node_response.timestamp,
			data: node_response.response,
		};

		cllctr.receive(noderes);
	}
}


#[derive(Clone, Debug)]
pub struct NodeResponse {
	nodeid: String,
	remote: IpAddr,
	timestamp: Timestamp,
	data: NodeData,
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


fn cmd_ls_nodes(_matches: clap::ArgMatches) {
	let db = sqlite::Connection::open(DATABASE_PATH).unwrap();
	let mut stmt = db.prepare("SELECT * FROM nodes").unwrap();
	let mut rows = stmt.query(NO_PARAMS).unwrap();

	let mut nodes = vec![];

	while let Some(row) = rows.next().unwrap() {
		nodes.push(Node::from_row(row));
	}

	json::to_writer_pretty(io::stdout(), &nodes).unwrap();
}



pub fn init_db() -> sqlite::Connection {
	let db = sqlite::Connection::open(DATABASE_PATH).unwrap();
	db.execute_batch(include_str!("./init_db.sql")).unwrap();
	db
}


fn clap_app<'a, 'b>() -> clap::App<'a, 'b> {
	clap::App::new(APPNAME)
		.version("0.0.0")
		.arg(
			clap::Arg::with_name("config")
				.short("c")
				.long("config")
				.help("custom config file")
				.takes_value(true)
				.validator(|x| match File::open(x) {
					Err(e) => Err(e.to_string()),
					Ok(_) => Ok(()),
				}),
		)
		.arg(clap::Arg::with_name("iface")
			.short("i")
			.long("iface")
			.help("respondd interface")
			.takes_value(true)
			.validator(|x| if_to_index(&x).map_or(Err("no interface".to_owned()), |_| Ok(()))),
		)
		.arg(
			clap::Arg::with_name("quiet")
				.short("q")
				.long("quiet")
				.help("disable output")
				.takes_value(false),
		)
		.arg(
			clap::Arg::with_name("interval")
				.long("interval")
				.help("multicast interval")
				.takes_value(true)
				.validator(|x| match x.parse::<u64>() {
					Ok(_) => Ok(()),
					Err(e) => Err(e.to_string()),
				}),
		)
		.subcommand(
			clap::SubCommand::with_name("collect").about("collect and save data")
		)
		.subcommand(
			clap::SubCommand::with_name("ls-nodes")
				.about("list all nodes")
		)
}

#[derive(Debug)]
pub enum Error {}

// impl From<influxdb::Error> for Error {
// 	fn from(e: influxdb::Error) -> Error {
// 		Error::Influx(e)
// 	}
// }
