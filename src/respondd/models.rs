use serde::{Serialize, Deserialize};
use serde_json as json;
use std::collections::HashMap;
use crate::NodeId;

#[derive(Serialize, Deserialize)]
pub struct Response {
	#[serde(rename = "neighbours")]
	neighbours: Neighbours,

	#[serde(rename = "nodeinfo")]
	nodeinfo: Nodeinfo,

	#[serde(rename = "statistics")]
	statistics: Statistics,
}

#[derive(Serialize, Deserialize)]
pub struct Neighbours {
	#[serde(rename = "batadv")]
	pub batadv: HashMap<String, BatAdvNeighbours>,

	#[serde(rename = "node_id")]
	node_id: NodeId,

	#[serde(rename = "wifi")]
	wifi: Wifi,
}

#[derive(Serialize, Deserialize)]
pub struct BatAdvNeighbours {
	#[serde(rename = "neighbours")]
	pub neighbours: HashMap<NodeId, BatAdvNeighbour>,
}


#[derive(Serialize, Deserialize)]
pub struct BatAdvNeighbour {
	#[serde(rename = "best")]
	pub best: bool,
	#[serde(rename = "lastseen")]
	pub lastseen: f64,
	#[serde(rename = "tq")]
	pub tq: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Wifi {}

#[derive(Serialize, Deserialize)]
pub struct Nodeinfo {
	#[serde(rename = "hardware")]
	hardware: Hardware,

	#[serde(rename = "hostname")]
	hostname: String,

	#[serde(rename = "location")]
	location: Location,

	#[serde(rename = "network")]
	network: Network,

	#[serde(rename = "node_id")]
	node_id: String,

	#[serde(rename = "owner")]
	owner: Owner,

	#[serde(rename = "software")]
	software: Software,

	#[serde(rename = "system")]
	system: System,
}

#[derive(Serialize, Deserialize)]
pub struct Hardware {
	#[serde(rename = "model")]
	model: String,

	#[serde(rename = "nproc")]
	nproc: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Location {
	#[serde(rename = "altitude")]
	altitude: f64,

	#[serde(rename = "latitude")]
	latitude: f64,

	#[serde(rename = "longitude")]
	longitude: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Network {
	#[serde(rename = "addresses")]
	addresses: Vec<String>,

	#[serde(rename = "mac")]
	mac: String,

	#[serde(rename = "mesh")]
	mesh: Mesh,
}

#[derive(Serialize, Deserialize)]
pub struct Mesh {
	#[serde(rename = "bat0")]
	bat0: Bat0,
}

#[derive(Serialize, Deserialize)]
pub struct Bat0 {
	#[serde(rename = "interfaces")]
	interfaces: Interfaces,
}

#[derive(Serialize, Deserialize)]
pub struct Interfaces {
	#[serde(rename = "other")]
	other: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Owner {
	#[serde(rename = "contact")]
	contact: String,
}

#[derive(Serialize, Deserialize)]
pub struct Software {
	#[serde(rename = "autoupdater")]
	autoupdater: Autoupdater,

	#[serde(rename = "batman-adv")]
	batman_adv: BatmanAdv,

	#[serde(rename = "fastd")]
	fastd: Fastd,

	#[serde(rename = "firmware")]
	firmware: Firmware,
}

#[derive(Serialize, Deserialize)]
pub struct Autoupdater {
	#[serde(rename = "branch")]
	branch: String,

	#[serde(rename = "enabled")]
	enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct BatmanAdv {
	#[serde(rename = "compat")]
	compat: i64,

	#[serde(rename = "version")]
	version: String,
}

#[derive(Serialize, Deserialize)]
pub struct Fastd {
	#[serde(rename = "enabled")]
	enabled: bool,

	#[serde(rename = "version")]
	version: String,
}

#[derive(Serialize, Deserialize)]
pub struct Firmware {
	#[serde(rename = "base")]
	base: String,

	#[serde(rename = "release")]
	release: String,
}

#[derive(Serialize, Deserialize)]
pub struct System {
	#[serde(rename = "domain_code")]
	domain_code: String,

	#[serde(rename = "primary_domain_code")]
	primary_domain_code: String,

	#[serde(rename = "site_code")]
	site_code: String,
}

#[derive(Serialize, Deserialize)]
pub struct Statistics {
	#[serde(rename = "clients")]
	clients: Clients,

	#[serde(rename = "gateway")]
	gateway: String,

	#[serde(rename = "gateway_nexthop")]
	gateway_nexthop: String,

	#[serde(rename = "idletime")]
	idletime: f64,

	#[serde(rename = "loadavg")]
	loadavg: f64,

	#[serde(rename = "memory")]
	memory: Memory,

	#[serde(rename = "node_id")]
	node_id: String,

	#[serde(rename = "processes")]
	processes: Processes,

	#[serde(rename = "rootfs_usage")]
	rootfs_usage: f64,

	#[serde(rename = "stat")]
	stat: Stat,

	#[serde(rename = "time")]
	time: i64,

	#[serde(rename = "traffic")]
	traffic: Traffic,

	#[serde(rename = "uptime")]
	uptime: f64,

	#[serde(rename = "wireless")]
	wireless: Vec<Wireless>,
}

#[derive(Serialize, Deserialize)]
pub struct Clients {
	#[serde(rename = "owe")]
	owe: i64,

	#[serde(rename = "owe24")]
	owe24: i64,

	#[serde(rename = "owe5")]
	owe5: i64,

	#[serde(rename = "total")]
	total: i64,

	#[serde(rename = "wifi")]
	wifi: i64,

	#[serde(rename = "wifi24")]
	wifi24: i64,

	#[serde(rename = "wifi5")]
	wifi5: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Memory {
	#[serde(rename = "available")]
	available: i64,

	#[serde(rename = "buffers")]
	buffers: i64,

	#[serde(rename = "cached")]
	cached: i64,

	#[serde(rename = "free")]
	free: i64,

	#[serde(rename = "total")]
	total: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Processes {
	#[serde(rename = "running")]
	running: i64,

	#[serde(rename = "total")]
	total: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Stat {
	#[serde(rename = "cpu")]
	cpu: Cpu,

	#[serde(rename = "ctxt")]
	ctxt: i64,

	#[serde(rename = "intr")]
	intr: i64,

	#[serde(rename = "processes")]
	processes: i64,

	#[serde(rename = "softirq")]
	softirq: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Cpu {
	#[serde(rename = "idle")]
	idle: i64,

	#[serde(rename = "iowait")]
	iowait: i64,

	#[serde(rename = "irq")]
	irq: i64,

	#[serde(rename = "nice")]
	nice: i64,

	#[serde(rename = "softirq")]
	softirq: i64,

	#[serde(rename = "system")]
	system: i64,

	#[serde(rename = "user")]
	user: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Traffic {
	#[serde(rename = "forward")]
	forward: Forward,

	#[serde(rename = "mgmt_rx")]
	mgmt_rx: Forward,

	#[serde(rename = "mgmt_tx")]
	mgmt_tx: Forward,

	#[serde(rename = "rx")]
	rx: Forward,

	#[serde(rename = "tx")]
	tx: Tx,
}

#[derive(Serialize, Deserialize)]
pub struct Forward {
	#[serde(rename = "bytes")]
	bytes: i64,

	#[serde(rename = "packets")]
	packets: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Tx {
	#[serde(rename = "bytes")]
	bytes: i64,

	#[serde(rename = "dropped")]
	dropped: i64,

	#[serde(rename = "packets")]
	packets: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Wireless {
	#[serde(rename = "active")]
	active: i64,

	#[serde(rename = "busy")]
	busy: i64,

	#[serde(rename = "frequency")]
	frequency: i64,

	#[serde(rename = "noise")]
	noise: i64,

	#[serde(rename = "rx")]
	rx: i64,

	#[serde(rename = "tx")]
	tx: i64,
}
