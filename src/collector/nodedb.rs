// SQL only allowed here!!!

use crate::NodeId;
use crate::NodeResponse;
use crate::CONFIG;
use chrono;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use rusqlite as sqlite;
use rusqlite::params;
use rusqlite::types::FromSqlError;
use rusqlite::NO_PARAMS;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{self, Display};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct NodeDb {
	db: Arc<Mutex<sqlite::Connection>>,
}

impl NodeDb {
	pub fn new(path: &str) -> Self {
		let db = sqlite::Connection::open(path).unwrap();
		db.execute_batch(include_str!("./init_db.sql")).unwrap();

		// disable synchronization. too slow
		db.pragma(None, "synchronous", &"OFF".to_string(), |_| Ok(())).unwrap();

		Self {
			db: Arc::new(Mutex::new(db)),
		}
	}

	pub fn get_node(&self, nodeid: &NodeId) -> Option<Node> {
		let node =
			self.db
				.lock()
				.unwrap()
				.query_row("SELECT * FROM nodes WHERE nodeid == ?1", params![nodeid], |row| {
					Ok(Node::from_row(row))
				});

		node.ok()
	}

	pub fn is_known(&self, nodeid: &NodeId) -> bool {
		let count: i64 = self
			.db
			.lock()
			.unwrap()
			.query_row("SELECT COUNT(*) FROM nodes WHERE nodeid = ?1", params![nodeid], |row| {
				row.get(0)
			})
			.unwrap();

		count == 1
	}

	pub fn update_node(&mut self, _nodedata: NodeResponse) {
		let _stmt = self
			.db
			.lock()
			.unwrap()
			.execute(
				"INSERT INTO nodes (nodeid, ) VALUES() nodes WHERE nodeid = ?1",
				params![],
			)
			.unwrap();
	}

	pub fn insert_node(&mut self, n: &NodeResponse) -> Option<()> {
		self.db
			.lock()
			.unwrap()
			.execute(
				"INSERT INTO nodes (nodeid, lastseen, firstseen, status, lastaddress, lastresponse)
				VALUES             (?1,     ?2,       ?2,        ?5,     ?3,          ?4)
				ON CONFLICT(nodeid) DO UPDATE SET
				(lastseen, lastaddress, status, lastresponse) =
				(?2,       ?3,          ?5,   ?4)",
				params![n.nodeid, n.timestamp, n.remote.to_string(), n.data, NodeStatus::Up],
			)
			.map_err(|e| {
				error!("sql error: {}", e);
			})
			.unwrap();

		Some(())
	}

	pub fn set_status(&mut self, id: &NodeId, status: NodeStatus) -> Result<usize, sqlite::Error> {
		self.db.lock().unwrap().execute(
			"UPDATE nodes  SET (status) = (?2) WHERE nodeid = ?1",
			params![id, status],
		)
	}

	pub fn get_all_nodes(&self) -> Vec<Node> {
		let db = self.db.lock().unwrap();
		let mut stmt = db.prepare("SELECT * FROM nodes").unwrap();

		stmt.query_map(NO_PARAMS, |row| Ok(Node::from_row(row)))
			.unwrap()
			.map(|n| n.unwrap())
			.collect()
	}

	pub fn get_all_nodes_with_status(&mut self, status: NodeStatus) -> Vec<Node> {
		let db = self.db.lock().unwrap();
		let mut stmt = db.prepare("SELECT * FROM nodes WHERE status=?1").unwrap();

		stmt.query_map(params![status], |row| Ok(Node::from_row(row)))
			.unwrap()
			.map(|n| n.expect("can't convert row to Node"))
			.collect()
	}

	pub fn delete_node(&mut self, nodeid: &NodeId) {
		let db = self.db.lock().unwrap();

		db.execute("DELETE FROM nodes WHERE nodeid=?1", params![nodeid])
			.unwrap();
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

	/// A node is `online` when it was recently seen.
	/// How long `recently` is, can be configured with `database.offline_after`
	pub fn is_online(&self) -> bool {
		self.since_last_seen() < Duration::seconds(CONFIG.database.offline_after as i64)
	}

	/// A node is offline, when it was not seen within the configure `database.offline_after` threshhold
	/// but the last message is not older than the `database.remove_after` duration.
	pub fn is_offline(&self) -> bool {
		!self.is_online() && !self.is_dead()
	}

	/// When a node wasn't seen for a very long time
	/// we consider it as `dead`. In this case the node should be forgotton and
	/// gets removed from the database
	pub fn is_dead(&self) -> bool {
		self.since_last_seen() > Duration::seconds(CONFIG.database.remove_after as i64)
	}

	pub fn since_last_seen(&self) -> Duration {
		Utc::now() - self.last_seen
	}
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
	Up,
	Down,
}

impl Display for NodeStatus {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Up => write!(f, "up"),
			Self::Down => write!(f, "down"),
		}
	}
}

impl FromStr for NodeStatus {
	type Err = String;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"up" => Ok(Self::Up),
			"down" => Ok(Self::Down),
			_ => Err("Unknown Status".to_string()),
		}
	}
}

#[test]
fn test_nodestatus_parsing() {
	assert_eq!(NodeStatus::from_str("up").unwrap(), NodeStatus::Up);
	assert_eq!(format!("{}", NodeStatus::Up), "up");
}

impl sqlite::types::FromSql for NodeStatus {
	fn column_result(value: sqlite::types::ValueRef<'_>) -> Result<Self, FromSqlError> {
		Ok(value.as_str()?.parse::<Self>().expect("cant parse nodestatus"))
	}
}

impl sqlite::ToSql for NodeStatus {
	fn to_sql(&self) -> Result<sqlite::types::ToSqlOutput, sqlite::Error> {
		Ok(sqlite::types::ToSqlOutput::Owned(sqlite::types::Value::Text(
			self.to_string(),
		)))
	}
}
