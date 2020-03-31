use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::ops;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hopglass2 {
	pub version: i64,
	pub nodes: Vec<Node>,
	pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
	pub nodeinfo: Nodeinfo,
	pub flags: Flags,
	pub statistics: Statistics,
	pub lastseen: DateTime<Utc>,
	pub firstseen: DateTime<Utc>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Nodeinfo {
	pub software: Option<Software>,
	pub network: Option<Network>,
	pub location: Option<Location>,
	pub owner: Option<Owner>,
	pub system: Option<System>,
	pub node_id: String,
	pub hostname: String,
	pub hardware: Option<Hardware>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Software {
	pub autoupdater: Autoupdater,
	#[serde(rename = "batman-adv")]
	pub batman_adv: BatmanAdv,
	pub fastd: Fastd,
	pub firmware: Firmware,
	#[serde(rename = "status-page")]
	pub status_page: Option<StatusPage>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Autoupdater {
	pub branch: String,
	pub enabled: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatmanAdv {
	pub version: String,
	pub compat: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fastd {
	pub version: String,
	pub enabled: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Firmware {
	pub base: String,
	pub release: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusPage {
	pub api: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Network {
	pub addresses: Vec<String>,
	pub mesh: Mesh,
	pub mac: String,
	pub mesh_interfaces: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mesh {
	pub bat0: Bat0,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bat0 {
	pub interfaces: Interfaces,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interfaces {
	#[serde(default)]
	pub wireless: Vec<String>,
	#[serde(default)]
	pub tunnel: Vec<String>,
	#[serde(default)]
	pub other: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
	pub latitude: f64,
	pub longitude: f64,
	pub altitude: Option<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Owner {
	pub contact: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct System {
	pub site_code: String,
	pub domain_code: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hardware {
	pub model: Option<String>,
	pub nproc: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flags {
	pub online: bool,
	pub uplink: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statistics {
	pub uptime: Option<u64>,
	pub gateway: Option<String>,
	pub gateway_nexthop: Option<String>,
	pub wireless: Option<Wireless>,
	pub memory_usage: Option<f64>,
	pub rootfs_usage: Option<f64>,
	pub clients: Option<i64>,
	pub loadavg: Option<f64>,
	pub traffic: Option<Traffic>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Traffic {
	pub forward: TrafficIO,
	pub mgmt_rx: TrafficIO,
	pub mgmt_tx: TrafficIO,
	pub rx: TrafficIO,
	pub tx: TrafficIO,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrafficIO {
	pub bytes: i64,
	pub packets: i64,
	pub dropped: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wireless {
	pub airtime2: Option<Airtime>,
	pub airtime5: Option<Airtime>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Airtime {
	pub rx: f64,
	pub tx: f64,
	pub wait: f64,
	pub free: f64,
}

impl ops::AddAssign for Traffic {
	fn add_assign(&mut self, other: Self) {
		*self = Self {
			forward: self.forward.clone() + other.forward,
			mgmt_rx: self.mgmt_rx.clone() + other.mgmt_rx,
			mgmt_tx: self.mgmt_tx.clone() + other.mgmt_tx,
			rx: self.rx.clone() + other.rx,
			tx: self.tx.clone() + other.tx,
		}
	}
}

impl Default for TrafficIO {
	fn default() -> Self {
		Self {
			bytes: 0,
			packets: 0,
			dropped: Some(0),
		}
	}
}

impl ops::Add for Traffic {
	type Output = Traffic;

	fn add(self, other: Self::Output) -> Self::Output {
		Self {
			forward: self.forward + other.forward,
			mgmt_rx: self.mgmt_rx + other.mgmt_rx,
			mgmt_tx: self.mgmt_tx + other.mgmt_tx,
			rx: self.rx + other.rx,
			tx: self.tx + other.tx,
		}
	}
}

impl ops::Add for TrafficIO {
	type Output = TrafficIO;

	fn add(self, other: Self::Output) -> Self::Output {
		Self {
			bytes: self.bytes + other.bytes,
			packets: self.packets + other.packets,
			dropped: Some(self.dropped.unwrap_or(0) + other.dropped.unwrap_or(0)),
		}
	}
}

impl ops::AddAssign for TrafficIO {
	fn add_assign(&mut self, other: Self) {
		*self = Self {
			bytes: self.bytes + other.bytes,
			packets: self.packets + other.packets,
			dropped: Some(self.dropped.unwrap_or(0) + other.dropped.unwrap_or(0)),
		}
	}
}
