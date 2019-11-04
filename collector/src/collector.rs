use chrono;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use crate::config;
use crate::DATABASE_PATH;
use crate::Error;
use crate::model;
use crate::multicast;
use crate::multicast::RecordType;
use crate::multicast::ResponddResponse;
use crate::output;
use crate::TABLE;
use eui48::MacAddress;
use influx_db_client as influxdb;
use influx_db_client::Point;
use influx_db_client::Value as Val;
use log::{debug, error, warn, info, trace};
use model::Response;
use postgres;
use reqwest;
use rusqlite as sqlite;
use serde_json as json;
use serde_json::Value;
use sqlite::params;
use std::collections::HashMap;
use std::ops;
use std::thread;
use std::time::Duration;
use crossbeam_channel::Receiver;
use crate::init_db;

pub mod db;


pub struct Collector {
	db: db::DB,
}



impl Collector {
	pub fn new() -> Self {
		Self {
			db: db::DB::new()
		}
	}

	pub fn recv_record(&self, rec: ResponddResponse) {
		self.db.save_raw_response(rec.clone()).unwrap();




	}
}






pub fn node_collector(config: &config::Config, receiver: Receiver<ResponddResponse>) -> Result<(), ()> {

	Err(())
}
