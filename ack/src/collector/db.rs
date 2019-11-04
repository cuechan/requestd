use chrono;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use crate::config;
use crate::DATABASE_PATH;
use crate::Error;
use crate::init_db;
use crate::model;
use crate::multicast;
use crate::multicast::RecordType;
use crate::multicast::ResponddResponse;
use crate::output;
use crate::TABLE;
use crossbeam_channel::Receiver;
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
use std::sync::{Arc, Mutex};
use rusqlite::NO_PARAMS;



type Nodeid = String;


pub struct DB {
	db: Arc<Mutex<sqlite::Connection>>,
}


impl DB {
	pub fn new() -> Self {
		let db = sqlite::Connection::open(DATABASE_PATH).unwrap();
		db.execute_batch(include_str!("../init_db.sql")).unwrap();

		Self {
			db: Arc::new(Mutex::new(db))
		}
	}



	pub fn save_raw_response(&self, response: ResponddResponse) -> Option<usize> {
		trace!("saving raw response");
		let changed = self.db.lock().unwrap().execute(
			"INSERT INTO raw_responses (timestamp, remote, response) VALUES (?, ?, ?)",
			&[response.timestamp.to_string(), response.remote.ip().to_string(), json::to_string(&response.response).unwrap()]
		).unwrap();

		Some(changed)
	}



	pub fn get_node(&self, nodeid: Nodeid) -> Node {
		let db = init_db();
		let mut stmt = db.prepare("SELECT * FROM nodes WHERE nodeid == ?1").unwrap();
		let mut rows = stmt.query(&[nodeid.clone()]).unwrap();

		while let Some(record) = rows.next().unwrap() {
			let node_response = ResponddResponse {
				timestamp: record.get("timestamp").unwrap(),
				remote: record.get::<_, String>("remote").unwrap().parse().unwrap(),
				response: json::from_str(&record.get::<_, String>("response").unwrap()).unwrap(),
			};
		}

		Node {
			nodeid: nodeid
		}
	}
}



pub struct Node {
	nodeid: Nodeid
}


impl Into<Nodeid> for Node {
	fn into(self) -> Nodeid {
		self.nodeid
	}
}



impl Node {
	fn new(id: Nodeid) -> Self {
		Self {
			nodeid: id
		}
	}
}
