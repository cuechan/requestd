use clap;
use std::fs::File;
use std::process;
use std::thread;
use std::time::Duration;
use log::{trace, debug, info, warn, error};
use pretty_env_logger;

pub mod config;
pub mod collector;

pub const APPNAME: &str = "ffhl-collector";

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
