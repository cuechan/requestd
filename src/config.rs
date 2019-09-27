use log::error;
use postgres::params::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::process;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub db: DbConfig,
    pub sources: Vec<Source>,
    pub respondd: Respondd,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbConfig {
    pub influx: String,
    pub postgres: String,
    pub user: String,
    pub password: String,
    pub database: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub graph_url: String,
    pub nodes_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Respondd {
    pub iface: String,
    pub timeout: u64,
    pub interval: u64,
}


impl Default for Respondd {
    fn default() -> Self {
        Self {
            iface: "bat0".to_owned(),
            timeout: 5,
            interval: 15,
        }
    }
}

impl Config {
    pub fn load_config(matches: &clap::ArgMatches) -> Self {
        let path = matches
            .value_of("config")
            .or(Some("./config.toml"))
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

        toml::from_str(&config_str).expect("something is wrong with your config")
    }
}

impl DbConfig {
    pub fn connection_params(&self) -> ConnectParams {
        ConnectParams::builder()
            .user(&self.user, Some(&self.password))
            .database(&self.database)
            .build(Host::Tcp(self.postgres.clone()))
    }
}
