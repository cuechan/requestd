extern crate clap;

pub mod config;
pub mod collector;

use std::fs::File;
use std::process;
use std::thread;
use std::time::Duration;

pub const APPNAME: &str = "ffhl-collector";

fn main() {

	// read config files
	let matches = load_args();
	let config = config::Config::load_config(&matches);


	if matches.is_present("dbsetup") {
		dbsetup(&config.db).unwrap();
		process::exit(0);
	}
	if matches.is_present("collect") {
		collector::collect(&config);
	}
}




fn load_args<'a>() -> clap::ArgMatches<'a> {
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
		.subcommand(clap::SubCommand::with_name("collect")
			.about("collect and save data")
		)
		.subcommand(clap::SubCommand::with_name("dbsetup")
			.about("collect and save data")
		)
		.get_matches()
}




fn dbsetup(config: &config::DbConfig) -> Result<(), ()> {
	let psql = postgres::Connection::connect(config.connection_params(), postgres::TlsMode::None).unwrap();

	eprintln!("THIS WILL DROP EXISTING TABLES! in 5s");
	thread::sleep(Duration::from_secs(5));

	eprintln!("creating tables");

	psql.batch_execute(
		"DROP TABLE IF EXISTS nodes;
		CREATE TABLE nodes (
			_id BIGSERIAL NOT NULL UNIQUE PRIMARY KEY,
			_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
			timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
			nodedata jsonb NOT NULL
		);"
	).unwrap();

	Ok(())
}
