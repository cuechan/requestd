// SQL only allowed here!!!

use chrono;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use crate::CONFIG;
use crate::DATABASE_PATH;
use crate::NodeResponse;
#[allow(unused_imports)]
use log::{debug, error, warn, info, trace};
use rusqlite as sqlite;
use rusqlite::params;
use rusqlite::NO_PARAMS;
use serde_json::Value;
// use std::rc::Rc;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use sqlite::types::FromSqlError;
use std::str::FromStr;
use std::fmt::{self, Display};

pub type NodeId = String;


#[derive(Clone)]
pub struct TriggerDb {
	db: Arc<Mutex<sqlite::Connection>>,
}


impl TriggerDb {
	pub fn new(db: Arc<Mutex<sqlite::Connection>>) -> Self {
		db.lock().unwrap().execute_batch(include_str!("../init_db.sql")).unwrap();

		db.lock().unwrap().pragma(None, "synchronous", &"OFF".to_string(), |_| Ok(())).unwrap();

		Self {
			db: db
		}
	}


	pub fn get_status(&self, nodeid: &NodeId) -> Vec<TriggerStatus> {
		let db = self.db.lock().unwrap();
		let mut stmt = db.prepare("SELECT * FROM trigger WHERE nodeid == ?1").unwrap();

		stmt.query_map(params![nodeid], |row| Ok(TriggerStatus::from_row(row)))
			.unwrap()
			.map(|n| n.unwrap())
			.collect()
	}




	pub fn trigger_is_present(&self, nodeid: &NodeId, trigger: String) -> bool {
		let count: i64 = self.db.lock().unwrap().query_row(
			"SELECT COUNT(*) FROM nodes WHERE nodeid = ?1",
			params![nodeid],
			|row| row.get(0)
		).unwrap();

		count == 1
	}


	pub fn set_trigger(&mut self, n: &NodeId, t: Trigger) -> Result<usize, sqlite::Error> {
		self.db.lock().unwrap().execute(
			"INSERT INTO trigger (nodeid, status)
			VALUES(?1, ?2)
			ON CONFLICT(nodeid, status) DO NOTHING",
			params![n, t]
		)
	}
}



pub struct TriggerStatus {
	nodeid: NodeId,
	status: Trigger,
}


impl TriggerStatus {
	pub fn from_row(row: &sqlite::Row) -> Self {
		Self {
			nodeid: row.get("nodeid").unwrap(),
			status: row.get("status").unwrap(),
		}
	}
}


#[derive(Clone, Debug, Copy)]
pub enum Trigger {
	NodeOffline,
	NodeOnline,
}



impl Display for Trigger {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}



impl FromStr for Trigger {
	type Err = String;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"NodeOffline" => Ok(Self::NodeOffline),
			"NodeOnline" => Ok(Self::NodeOnline),
			_ => Err("Unknown Trigger".to_string())
		}
	}
}



impl sqlite::types::FromSql for Trigger {
	fn column_result(value: sqlite::types::ValueRef<'_>) -> Result<Self, FromSqlError> {
		value.as_str()?.parse::<Self>().or(Err(FromSqlError::InvalidType))
	}
}


impl sqlite::ToSql for Trigger {
	fn to_sql(&self) -> Result<sqlite::types::ToSqlOutput, sqlite::Error> {
		Ok(sqlite::types::ToSqlOutput::Owned(
			sqlite::types::Value::Text(self.to_string())
		))
	}
}
