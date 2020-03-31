use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::process;
use serde;
use serde_yaml as yaml;
use crate::DEFAULT_CONF_FILE;
use crate::DEFAULT_MIN_ACTIVE;
use crate::DEFAULT_OFFLINE_THRESH;
use std::collections::HashMap;
use log::{error, warn, info, debug, trace};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
	pub database: DbConfig,
	pub respondd: Respondd,
	pub events: Events,
}

impl Config {
	pub fn load_config(matches: &clap::ArgMatches) -> Result<Self, yaml::Error> {
		let path = matches
			.value_of("config")
			.or(Some(DEFAULT_CONF_FILE))
			.unwrap();

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

		let mut conf: Self = yaml::from_str(&config_str)?;

		if let Some(interval) = matches.value_of("interval") {
			conf.respondd.interval = interval.parse().unwrap();
		}

		if let Some(iface) = matches.value_of("iface") {
			conf.respondd.interface = iface.to_owned();
		}


		if !(conf.database.offline_thresh > conf.respondd.interval) {
			warn!("`database.offline_thresh` should be greate than `respondd.interval`");
		}


		Ok(conf)
	}
}



impl Default for Config {
	fn default() -> Self {
		Self {
			database: DbConfig::default(),
			respondd: Respondd::default(),
			events: Events::default(),
		}
	}
}



#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DbConfig {
	pub dbfile: String,
	pub min_active: u64,
	pub offline_thresh: u64,
	pub evaluate_every: u64,
}


impl Default for DbConfig {
	fn default() -> Self {
		Self {
			dbfile: "./nodes.db".to_string(),
			min_active: DEFAULT_MIN_ACTIVE,
			offline_thresh: DEFAULT_OFFLINE_THRESH,
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
	#[serde(rename = "multicastaddress")]
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
				"neighbours".to_string()
			],
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Events {
	pub new_node: Vec<Event>,
	pub node_offline: Vec<Event>,
	pub update: Vec<Event>,
	pub online_after_offline: Vec<Event>,
}

impl Default for Events {
	fn default() -> Self {
		Self {
			new_node: vec![],
			node_offline: vec![],
			update: vec![],
			online_after_offline: vec![],
		}
	}
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
	pub exec: String,
	#[serde(default)]
	pub vars: HashMap<String, String>
}
