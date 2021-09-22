use crate::DEFAULT_CONF_FILES;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use serde;
use serde::{Deserialize, Serialize};
use serde_yaml as yaml;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::process;
use std::path;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
	pub requestd: Requestd,
	pub web: Option<WebEndpoint>,
	pub mqtt: Option<MqttEndpoint>,
}

impl Config {
	pub fn load_config() -> Result<Self, yaml::Error> {
		let path = match get_first_file_found(DEFAULT_CONF_FILES) {
			Some(c) => c,
			None => {
				warn!("no config found. expected in some of these locations: {:?}", DEFAULT_CONF_FILES);
				process::exit(1);
			}
		};

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
			requestd: Requestd::default(),
			web: Some(WebEndpoint::default()),
			mqtt: None,
		}
	}
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Requestd {
	pub interface: String,
	pub interval: u64,
	pub multicast_address: String,
	pub categories: Vec<String>,
	pub clean_interval: u64,
	pub retention: u64,
}

impl Default for Requestd {
	fn default() -> Self {
		Self {
			interface: "bat0".to_owned(),
			interval: 60,
			// retention: 60*24*72, // retention of 3 days
			retention: 10, // retention of 3 days
			clean_interval: 10, // check for invalid responses every 2 minutes
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
#[serde(deny_unknown_fields)]
pub struct Event {
	pub exec: String,
	#[serde(default)]
	pub vars: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebEndpoint {
	pub listen: SocketAddr,
}

impl Default for WebEndpoint {
	fn default() -> Self {
		Self {
			listen: SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 21001)
		}
	}
}


#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MqttEndpoint {
	pub broker: String,
	pub topic: String,
}

impl Default for MqttEndpoint {
	fn default() -> Self {
		Self {
			broker: "localhost:1883".to_string(),
			topic: "requestd/responses".to_string(),
		}
	}
}
