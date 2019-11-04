use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use serde_json::Value;
use serde;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::ops;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Response {
	#[serde(rename = "nodeinfo")]
	Nodeinfo(Nodeinfo),
	#[serde(rename = "statistics")]
	Statistics(Statistics),
	#[serde(rename = "neighbours")]
	Neighbors(Neighbours),
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Neighbours {

}



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Nodeinfo {
    pub node_id: String,
    pub software: Option<Software>,
    pub network: Option<Network>,
    pub location: Option<Location>,
    pub owner: Option<Owner>,
    pub system: Option<System>,
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
    pub uptime: Option<f64>,
    pub gateway: Option<String>,
    pub gateway_nexthop: Option<String>,
    pub wireless: Option<Wireless>,
    pub memory_usage: Option<f64>,
    pub rootfs_usage: Option<f64>,
    pub clients: Option<ClientsDetailed>,
    pub loadavg: Option<f64>,
    pub traffic: Option<Traffic>,
}



#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientsDetailed {
    total: i64,
    wifi: i64,
    wifi24: i64,
    wifi5: i64,
}



#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Traffic {
    pub forward: TrafficIO,
    pub mgmt_rx: TrafficIO,
    pub mgmt_tx: TrafficIO,
    pub rx: TrafficIO,
    pub tx: TrafficIO,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
