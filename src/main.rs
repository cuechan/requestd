use clap;
use std::fs::File;
use std::process;
use std::thread;
use std::time::Duration;
use log::{trace, debug, info, warn, error};
use pretty_env_logger;
use postgres;
use influx_db_client as influxdb;
use rusqlite as sqlite;
use chrono::{DateTime, Utc};

pub mod config;
pub mod collector;

pub const APPNAME: &str = "ffhl-collector";
pub const TABLE: &str = "nodes";
pub const SQLITEDB: &str = "ffhl.db";
pub const DBDIR: &str = "db/";

fn main() {

	// read config files
	let mut clap = clap_app();
	let matches = clap.clone().get_matches();

	if !matches.is_present("quiet") {
		pretty_env_logger::init();
	}

	let config = config::Config::load_config(&matches);



	match matches.subcommand_name() {
		Some("dbsetup") => {
			dbsetup(&config.db).unwrap();
			process::exit(0);
		},
		Some("convert") => {
			convert_to_influx(&config);
			process::exit(0);
		},
		None | Some("collect") => {
			let sleep: u64 = matches.value_of("delay").unwrap().parse().unwrap();
			thread::sleep(Duration::from_millis(sleep));
			collector::collect(&config);
		},
		_ => {
			println!("foo");
			clap.print_help();
			process::exit(1);
		}
	}
}




pub fn convert_to_influx(config: &config::Config) {
	// let influx = influxdb::Client::new(
	// 	format!( "http://{}:{}", config.db.host, 8086),
	// 	config.db.database.clone()
	// );

	debug!("open/create sqlite database");
	// let sqlite = rusqlite::Connection::open_in_memory().unwrap();

	let psql = postgres::Connection::connect(
		config.db.connection_params(),
		postgres::TlsMode::None
	).unwrap();


	let mut offset = 0;
	let limit = 500;

	loop {
		let rows = psql.query(&format!("
			SELECT data, timestamp
			FROM {0}
			LIMIT {1}
			OFFSET {2}", TABLE, limit, offset),
			&[]
		).unwrap();


		let count = rows.len();

		for (i, row) in rows.into_iter().enumerate() {
			let value: serde_json::Value = row.get("data");
			let time_tz: DateTime<Utc> = row.get("timestamp");

			let nodeid = value.as_object()
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

			let sqlite = rusqlite::Connection::open(format!("{}/node-{}.db", DBDIR, nodeid)).unwrap();

			sqlite.execute(&format!("
				CREATE TABLE IF NOT EXISTS {0} (
					timestamp INTEGER NOT NULL,
					data TEXT NOT NULL
				);", TABLE),
				sqlite::NO_PARAMS
			).unwrap();




			sqlite.execute("
				INSERT INTO nodes (timestamp, data)
				VALUES (?1, ?2)",
				&[&time_tz.timestamp(), &serde_json::to_string(&value).unwrap() as &dyn sqlite::ToSql]
			).unwrap();

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
		);", TABLE
	)).unwrap();

	Ok(())
}





fn clap_app<'a, 'b>() -> clap::App<'a, 'b> {
	clap::App::new(APPNAME)
		.version("0.0.0")
		.arg(clap::Arg::with_name("config")
			.short("c")
			.long("config")
			.help("custom config file")
			.takes_value(true)
			.validator(|x| {
				match File::open(x) {
					Err(e) => Err(e.to_string()),
					Ok(_) => Ok(())
				}
			})
		)
		.arg(clap::Arg::with_name("quiet")
			.short("q")
			.long("quiet")
			.help("disable output")
			.takes_value(false)
		)
		.arg(clap::Arg::with_name("delay")
			.short("d")
			.long("delay")
			.help("delay before fetching data in ms")
			.takes_value(true)
			.default_value("0")
			.validator(|x| {
				match x.parse::<u64>() {
					Ok(_) => Ok(()),
					Err(e) => Err(e.to_string()),
				}
			})
		)
		.subcommand(clap::SubCommand::with_name("collect")
			.about("collect and save data")
		)
		.subcommand(clap::SubCommand::with_name("dbsetup")
			.about("delete old and create new tables")
		)
		.subcommand(clap::SubCommand::with_name("convert")
			.about("copy data from postgresql to influx")
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
