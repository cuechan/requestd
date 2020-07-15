use crate::DEFAULT_CONF_FILES;
use crate::DEFAULT_OFFLINE_THRESH;
use crate::DEFAULT_REMOVE_THRESH;
use log::{debug, error, info, trace, warn};
use serde;
use serde::{Deserialize, Serialize};
use serde_yaml as yaml;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::process;
// use std::fs::File;
use std::path;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
	pub database: DbConfig,
	pub respondd: Respondd,
	pub events: Events,
	pub controlsocket: String,
	pub concurrent_hooks: u64,
}

impl Config {
	pub fn load_config(matches: &clap::ArgMatches) -> Result<Self, yaml::Error> {
		let path = matches
			.value_of("config")
			.or(get_first_file_found(DEFAULT_CONF_FILES))
			.expect("no config found");

		let mut config_str = String::new();
		match File::open(path) {
			Err(e) => {
				eprintln!("no config file");
				error!("{}: {}", e, path);
				process::exit(1);
			}
			Ok(mut r) => {
				r.read_to_string(&mut config_str).unwrap();
			}
		}

		let conf: Self = yaml::from_str(&config_str)?;

		if conf.database.offline_after < conf.respondd.interval {
			warn!("`database.offline_after` should be greater than `respondd.interval`");
		}

		Ok(conf)
	}
}

fn get_first_file_found<'a>(files: &[&'a str]) -> Option<&'a str> {
	for file in files {
		if path::Path::new(file).is_file() {
			return Some(file);
		}
	}

	None
}

impl Default for Config {
	fn default() -> Self {
		Self {
			database: DbConfig::default(),
			respondd: Respondd::default(),
			events: Events::default(),
			controlsocket: "/tmp/requestd.sock".to_string(),
			concurrent_hooks: 4,
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DbConfig {
	pub dbfile: String,
	pub offline_after: u64,
	pub remove_after: u64,
	pub evaluate_every: u64,
}

impl Default for DbConfig {
	fn default() -> Self {
		Self {
			dbfile: "./nodes.db".to_string(),
			offline_after: DEFAULT_OFFLINE_THRESH,
			remove_after: DEFAULT_REMOVE_THRESH,
			evaluate_every: 15,
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Respondd {
	pub interface: String,
	pub timeout: u64,
	pub interval: u64,
	pub multicast_address: String,
	pub categories: Vec<String>,
}

impl Default for Respondd {
	fn default() -> Self {
		Self {
			interface: "bat0".to_owned(),
			timeout: 5,
			interval: 15,
			multicast_address: "ff05::2:1001".to_string(),
			categories: vec![
				"nodeinfo".to_string(),
				"statistics".to_string(),
				"neighbours".to_string(),
			],
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Events {
	pub new_node: Vec<Event>,
	pub node_offline: Vec<Event>,
	pub node_update: Vec<Event>,
	pub online_after_offline: Vec<Event>,
}

impl Default for Events {
	fn default() -> Self {
		Self {
			new_node: vec![],
			node_offline: vec![],
			node_update: vec![],
			online_after_offline: vec![],
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
	pub exec: String,
	#[serde(default)]
	pub vars: HashMap<String, String>,
}
