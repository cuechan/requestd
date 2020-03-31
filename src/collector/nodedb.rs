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
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use std::fmt::{self, Display};
use std::str::FromStr;
use rusqlite::types::FromSqlError;
use sqlite::types::ToSqlOutput;


pub type NodeId = String;


#[derive(Clone)]
pub struct NodeDb {
	db: Arc<Mutex<sqlite::Connection>>,
}


impl NodeDb {
	pub fn new() -> Self {
		let db = sqlite::Connection::open(DATABASE_PATH).unwrap();
		db.execute_batch(include_str!("../init_db.sql")).unwrap();

		db.pragma(None, "synchronous", &"OFF".to_string(), |_| Ok(())).unwrap();

		Self {
			db: Arc::new(Mutex::new(db))
		}
	}


	pub fn get_node(&self, nodeid: &NodeId) -> Option<Node> {
		let node = self.db.lock().unwrap().query_row(
			"SELECT * FROM nodes WHERE nodeid == ?1",
			params![nodeid],
			|row| Ok(Node::from_row(row))
		);

		node.ok()
	}


	pub fn is_known(&self, nodeid: &NodeId) -> bool {
		let count: i64 = self.db.lock().unwrap().query_row(
			"SELECT COUNT(*) FROM nodes WHERE nodeid = ?1",
			params![nodeid],
			|row| row.get(0)
		).unwrap();

		count == 1
	}


	pub fn update_node(&mut self, _nodedata: NodeResponse) {
		let _stmt = self.db.lock().unwrap().execute(
			"INSERT INTO nodes (nodeid, ) VALUES() nodes WHERE nodeid = ?1",
			params![]
		).unwrap();
	}


	pub fn insert_node(&mut self, n: &NodeResponse) -> Option<()> {
		self.db.lock().unwrap().execute(
			"INSERT INTO nodes (nodeid, lastseen, firstseen, status, lastaddress, lastresponse)
			VALUES             (?1,     ?2,       ?2,        'Up',   ?3,          ?4)
			ON CONFLICT(nodeid) DO UPDATE SET
			(lastseen, lastaddress, status, lastresponse) =
			(?2,       ?3,          'Up',   ?4)",
			params![n.nodeid, n.timestamp, n.remote.to_string(), n.data]
		).map_err(|e| {
			error!("sql error: {}", e);
		}).unwrap();

		Some(())
	}


	pub fn set_status(&mut self, id: NodeId, status: NodeStatus ) -> Result<usize, sqlite::Error> {
		self.db.lock().unwrap().execute(
			"UPDATE nodes  SET (status) = (?2) WHERE nodeid = ?1",
			params![id, status]
		)
	}


	pub fn get_all_nodes(&mut self) -> Vec<Node> {
		let mut db = self.db.lock().unwrap();
		let mut stmt = db.prepare("SELECT * FROM nodes").unwrap();

		stmt.query_map(NO_PARAMS, |row| Ok(Node::from_row(row)))
			.unwrap()
			.map(|n| n.unwrap())
			.collect()
	}
}


#[derive(Clone, Debug, Serialize)]
pub struct Node {
	pub nodeid: NodeId,
	pub last_seen: DateTime<Utc>,
	pub first_seen: DateTime<Utc>,
	pub last_address: String,
	pub last_response: Value,
	pub status: NodeStatus,
}


impl Into<NodeId> for Node {
	fn into(self) -> NodeId {
		self.nodeid
	}
}


impl Node {
	pub fn from_row(row: &sqlite::Row) -> Self {
		Self {
			nodeid: row.get("nodeid").unwrap(),
			last_seen: row.get("lastseen").unwrap(),
			first_seen: row.get("firstseen").unwrap(),
			last_address: row.get("lastaddress").unwrap(),
			last_response: row.get("lastresponse").unwrap(),
			status: row.get("status").unwrap(),
		}
	}

	/// The node was seen in the last `MIN_ACTIVE` hours
	pub fn is_active(&self) -> bool {
		(self.last_seen + Duration::seconds(CONFIG.database.min_active as i64)) > Utc::now()
	}

	/// was the node seen within the threshhold
	pub fn is_online(&self) -> bool {
		(self.last_seen + Duration::seconds(CONFIG.database.offline_thresh as i64)) > Utc::now()
	}


	pub fn is_offline(&self) -> bool {
		!self.is_online()
	}
}



#[derive(Clone, Debug, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
	Up,
	Down,
}


impl Display for NodeStatus {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}



impl FromStr for NodeStatus {
	type Err = String;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Up" => Ok(Self::Up),
			"Down" => Ok(Self::Down),
			_ => Err("Unknown Status".to_string())
		}
	}
}



impl sqlite::types::FromSql for NodeStatus {
	fn column_result(value: sqlite::types::ValueRef<'_>) -> Result<Self, FromSqlError> {
		value.as_str()?.parse::<Self>().or(Err(FromSqlError::InvalidType))
	}
}


impl sqlite::ToSql for NodeStatus {
	fn to_sql(&self) -> Result<sqlite::types::ToSqlOutput, sqlite::Error> {
		Ok(sqlite::types::ToSqlOutput::Owned(
			sqlite::types::Value::Text(self.to_string())
		))
	}
}
