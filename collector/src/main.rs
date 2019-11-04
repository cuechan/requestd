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

pub const APPNAME: &str = "ffhl-collector";
pub const TABLE: &str = "nodes";
pub const DATABASE_PATH: &str = "./nodes.db";


fn main() {
	// read config files
	let mut clap = clap_app();
	let matches = clap.clone().get_matches();

	if !matches.is_present("quiet") {
		pretty_env_logger::init();
	}

	let mut config = config::Config::load_config(&matches);

	if let Some(iface) = matches.value_of("iface") {
		config.respondd.iface = iface.to_owned();
	}



	println!("{:#?}", config);

	match matches.subcommand() {
		("replay", m) => {
			cmd_replay();
			process::exit(0);
		},
		("foreach", m) => {
			cmd_foreach(m.unwrap().clone());
		},
		("collect", m) => {
			cmd_collect(matches, &config);
		}
		_ => {
			println!("foo");
			clap.print_help().unwrap();
			process::exit(1);
		}
	}
}




fn cmd_replay() {
	let db = init_db();
	let mut stmt = db.prepare("SELECT * FROM raw_responses").unwrap();
	let mut rows = stmt.query(NO_PARAMS).unwrap();

	let cllctr = Collector::new();


	while let Some(record) = rows.next().unwrap() {
		let node_response = ResponddResponse {
			timestamp: record.get("timestamp").unwrap(),
			remote: record.get::<_, String>("remote").unwrap().parse().unwrap(),
			response: json::from_str(&record.get::<_, String>("response").unwrap()).unwrap(),
		};

		// trace!("deserializing: {:#?}", node_response.response);
		// let res: model::Response = json::from_value(node_response.response).unwrap();

		cllctr.recv_record(node_response);
	}
}


fn cmd_collect(m: clap::ArgMatches, config: &config::Config) {
	let db = init_db();
	let requester = multicast::ResponderService::start(&config.respondd.iface, config.respondd.interval);
	let receiver = requester.get_receiver();

	thread::spawn(move || {
		loop {
			info!("request new data");
			requester.request(RecordType::Statisitcs);
			requester.request(RecordType::Nodeinfo);
			requester.request(RecordType::Neighbours);

			thread::sleep(Duration::from_secs(30));
		}
	});


	let cllctr = Collector::new();


	for node_response in receiver {
		cllctr.recv_record(node_response.clone());
		// trace!("deserializing: {:#?}", node_response.response);
		// let res: model::Response = json::from_value(node_response.response).unwrap();

		cllctr.recv_record(node_response);
	}
}




fn cmd_foreach(matches: clap::ArgMatches) {
	let db = sqlite::Connection::open(DATABASE_PATH).unwrap();
	let mut stmt = db.prepare("SELECT * FROM raw_responses").unwrap();
	let mut rows = stmt.query(NO_PARAMS).unwrap();

	trace!("executing {:#?}", matches.value_of("command"));

	let cmd = matches.value_of("command").unwrap();

	while let Some(row) = rows.next().unwrap() {
		let timestamp: i64 = row.get("timestamp").unwrap();
		let remote: String = row.get("remote").unwrap();
		let data: String = row.get("response").unwrap();

		let mut child = process::Command::new("sh")
			.arg("-c")
			.arg(cmd)
			.stdin(Stdio::piped())
			.spawn()
			.unwrap();

		let mut stdin = child.stdin.as_mut().unwrap();
		stdin.write_all(data.as_bytes()).unwrap();

		let result = child.wait().unwrap();

		if !result.success() {
			println!("command exited with non-zero. Stopping here");
			process::exit(result.code().unwrap());
		}

	}
}



pub fn init_db() -> sqlite::Connection {
	let db = sqlite::Connection::open(DATABASE_PATH).unwrap();
	db.execute_batch(include_str!("./init_db.sql")).unwrap();
	db
}



pub fn convert_to_influx(config: &config::Config) {
	// let influx = influxdb::Client::new(
	// 	format!( "http://{}:{}", config.db.host, 8086),
	// 	config.db.database.clone()
	// );

	debug!("open/create sqlite database");
	// let sqlite = rusqlite::Connection::open_in_memory().unwrap();

	let psql =
		postgres::Connection::connect(config.db.connection_params(), postgres::TlsMode::None)
			.unwrap();

	let mut offset = 0;
	let limit = 500;

	loop {
		let rows = psql
			.query(
				&format!(
					"
			SELECT data, timestamp
			FROM {0}
			LIMIT {1}
			OFFSET {2}",
					TABLE, limit, offset
				),
				&[],
			)
			.unwrap();

		let count = rows.len();

		for (i, row) in rows.into_iter().enumerate() {
			let value: serde_json::Value = row.get("data");
			let time_tz: DateTime<Utc> = row.get("timestamp");

			let nodeid = value
				.as_object()
				.unwrap()
				.get("nodeinfo")
				.unwrap()
				.as_object()
				.unwrap()
				.get("node_id")
				.unwrap()
				.as_str()
				.unwrap()
				.to_string();

			let sqlite =
				rusqlite::Connection::open(format!("{}/node-{}.db", DATABASE_PATH, nodeid)).unwrap();

			sqlite
				.execute(
					&format!(
						"
				CREATE TABLE IF NOT EXISTS {0} (
					timestamp INTEGER NOT NULL,
					data TEXT NOT NULL
				);",
						TABLE
					),
					sqlite::NO_PARAMS,
				)
				.unwrap();

			sqlite
				.execute(
					"
				INSERT INTO nodes (timestamp, data)
				VALUES (?1, ?2)",
					&[
						&time_tz.timestamp(),
						&serde_json::to_string(&value).unwrap() as &dyn sqlite::ToSql,
					],
				)
				.unwrap();

			if i % 10 == 0 {
				info!("converting... {}/{}", i, count);
			}
		}

		if count < limit {
			println!("finished");
			break;
		}

		offset += limit;
	}

	// for (i, row) in rows.into_iter().enumerate() {
	// 	let value = row.get("data");
	// 	let time_tz = row.get("timestamp");

	// 	if i % 10 == 0 {
	// 		info!("converting... {}%", (i/count) as f64 * 100 as f64);
	// 	}

	// 	collector::store_node_influx(&influx, time_tz, &value).unwrap();
	// }
}

fn dbsetup(config: &config::DbConfig) -> Result<(), ()> {
	let psql = postgres::Connection::connect(config.connection_params(), postgres::TlsMode::None).unwrap();

	eprintln!("THIS WILL DROP EXISTING TABLES! in 5s");
	thread::sleep(Duration::from_secs(5));

	eprintln!("creating tables");

	psql.batch_execute(&format!(
		"DROP TABLE IF EXISTS {0};
		CREATE TABLE {0} (
			_id BIGSERIAL NOT NULL UNIQUE PRIMARY KEY,
			_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
			timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
			data jsonb NOT NULL
		);",
		TABLE
	))
	.unwrap();

	Ok(())
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
		.subcommand(
			clap::SubCommand::with_name("collect").about("collect and save data")
		)
		.subcommand(
			clap::SubCommand::with_name("foreach")
				.about("execute a command for every collected datapoint")
				.arg(clap::Arg::with_name("command")
					.help("command to execute")
					.required(true)
					.takes_value(true)
				)
		)
		.subcommand(
			clap::SubCommand::with_name("replay").about("recollects all data as it was just retrieved"),
		)
		.subcommand(
			clap::SubCommand::with_name("convert").about("copy data from postgresql to influx"),
		)
}

#[derive(Debug)]
pub enum Error {
	Influx(influxdb::Error),
}

// impl From<influxdb::Error> for Error {
// 	fn from(e: influxdb::Error) -> Error {
// 		Error::Influx(e)
// 	}
// }
