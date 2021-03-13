#![feature(proc_macro_hygiene, decl_macro)]

pub mod collector;
pub mod config;
pub mod controlsocket;
pub mod model;
pub mod monitor;
pub mod multicast;
pub mod output;
pub mod web;

use chrono::{DateTime, Utc};
use clap;
use collector::nodedb;
use collector::Collector;
use config::Config;
use lazy_static::lazy_static;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use monitor::metrics;
use nodedb::Node;
use nodedb::NodeDb;
use pretty_env_logger;
use rusqlite as sqlite;
use rusqlite::NO_PARAMS;
use serde_json as json;
use serde_yaml as yaml;
use std::fs::File;
use std::io;
use std::net::IpAddr;
use std::process;
use std::process::exit;
use std::thread;
use std::time::Duration;
use rocket::{self, get, routes};
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::time;
use serde_json::json;


pub const APPNAME: &str = "ffhl-collector";
pub const TABLE: &str = "nodes";
pub const DEFAULT_CONF_FILES: &[&str] = &["/etc/requestd.yml", "./config.yml"];

pub type NodeData = json::Value;
pub type Timestamp = DateTime<Utc>;
pub type NodeId = String;

lazy_static! {
	pub static ref ARGS: clap::ArgMatches<'static> = clap_app().get_matches();
	pub static ref CONFIG: Config = {
		config::Config::load_config(&ARGS)
			.map_err(|e| {
				error!("loading config: {}", e);
				exit(1);
			})
			.unwrap()
	};
}


fn main() {
	pretty_env_logger::init();

	trace!("config: \n{}", yaml::to_string(&*CONFIG).unwrap());

	match ARGS.subcommand() {
		("ls-nodes", m) => {
			cmd_ls_nodes(m.unwrap().clone());
		}
		("collect", _m) => {
			cmd_collect();
		}
		("foo", m) => {
			web_test(m.unwrap())
		}
		_ => {
			error!("not a valid Command. Try --help");
			process::exit(1);
		}
	}
}

fn web_test(args: &clap::ArgMatches) {
	let db = NodeDb::new(&CONFIG.database.dbfile);
}

// TODO: this needs a bit more/clearer structure
fn cmd_collect() {
	monitor::start_exporter();

	let requester = multicast::RequesterService::new(&CONFIG.respondd.interface);
	let receiver = requester.get_receiver();
	let db = NodeDb::new(&CONFIG.database.dbfile);

	// start the controlsocket listener
	let db_c = db.clone();
	std::thread::spawn(move || {
		controlsocket::start(db_c, &CONFIG.controlsocket);
	});


	let mut cllctr = Collector::new(db.clone());
	cllctr.start_collector(requester.clone());

	let db_c = db.clone();
	let cllctr_c = cllctr.clone();
	std::thread::spawn(move || {
		web::main(db_c, cllctr_c);
	});

	thread::spawn(move || loop {
		debug!("requesting new data");
		requester.request(&CONFIG.respondd.multicast_address, &CONFIG.respondd.categories);

		thread::sleep(Duration::from_secs(CONFIG.respondd.interval));
	});

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
	nodeid: NodeId,
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
	let db = sqlite::Connection::open(&CONFIG.database.dbfile).unwrap();
	let mut stmt = db.prepare("SELECT * FROM nodes").unwrap();
	let mut rows = stmt.query(NO_PARAMS).unwrap();

	let mut nodes = vec![];

	while let Some(row) = rows.next().unwrap() {
		nodes.push(Node::from_row(row));
	}

	json::to_writer_pretty(io::stdout(), &nodes).unwrap();
}

fn clap_app<'a, 'b>() -> clap::App<'a, 'b> {
	clap::App::new(env!("CARGO_PKG_NAME"))
		.version(env!("CARGO_PKG_VERSION"))
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
		.subcommand(clap::SubCommand::with_name("collect").about("collect and save data"))
		.subcommand(clap::SubCommand::with_name("ls-nodes").about("list all nodes"))
		.subcommand(clap::SubCommand::with_name("foo").about("do foo things"))
}

#[derive(Debug)]
pub enum Error {}

// impl From<influxdb::Error> for Error {
// 	fn from(e: influxdb::Error) -> Error {
// 		Error::Influx(e)
// 	}
// }
