use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Read;
use postgres::params::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
	pub db: DbConfig,
	pub sources: SourcesConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbConfig {
	pub host: String,
	pub port: u16,
	pub user: String,
	pub password: String,
	pub database: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourcesConfig {
	pub graph_url: String,
	pub nodes_url: String,
}



impl Config {
	pub fn load_config(matches: &clap::ArgMatches) -> Self {
		let path = matches.value_of("config")
			.or(Some("./config.toml"))
			.unwrap();

		let mut config_str = String::new();
		File::open(path).expect(&format!("there is no config file at {}", path))
			.read_to_string(&mut config_str)
			.unwrap();


		toml::from_str(&config_str).expect("something is wrong with your config")
	}
}


impl DbConfig {
	pub fn connection_params(&self) -> ConnectParams {
		ConnectParams::builder()
			.user(&self.user, Some(&self.password))
			.port(self.port)
			.database(&self.database)
			.build(Host::Tcp(self.host.clone()))
	}
}
