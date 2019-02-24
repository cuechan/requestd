use crate::config;
use reqwest;
use serde_json;
use postgres;
use chrono;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use influx_db_client as influxdb;
use influx_db_client::Value as Val;


pub fn collect(config: &config::Config) -> Result<(), ()> {
	let graphs: serde_json::Value = reqwest::get(&config.sources.graph_url).unwrap().json().unwrap();
	let nodes: serde_json::Value = reqwest::get(&config.sources.nodes_url).unwrap().json().unwrap();



	// "%Y-%m-%dT%H:%M:%S",

	let time: NaiveDateTime = nodes.as_object()
			.unwrap()
			.get("timestamp")
			.unwrap()
			.as_str()
			.unwrap()
			.parse()
			.unwrap();

	let time_z = DateTime::<Utc>::from_utc(time, Utc);
	println!("{}", time_z);


	let host = format!("http://{}:8086/", "10.8.1.1");
	let flux = influxdb::Client::new(host, "ffhl-nodes".to_string());


	let _measurements = Vec::<model::Node>::new();
	for (i, node_val) in nodes.as_object().unwrap().get("nodes").unwrap().as_array().unwrap().iter().enumerate() {
		let node: model::Node = match serde_json::from_value(node_val.clone()) {
			Err(e) => {
				eprintln!("{:#?}", node_val);
				panic!(e.to_string());
			},
			Ok(r) => r
		};


		let mut data = influxdb::keys::Point::new("node");
		node.get_measurement(&mut data);
		data.add_timestamp(time_z.timestamp_millis());


		println!("{:#?}", data);

		print!("Nodes: {:03}\r", i);

		flux.write_point(data, Some(influxdb::keys::Precision::Milliseconds), None).unwrap();
	}

	Err(())
}




pub mod model {
	use serde_json::Value;
	use serde::{Serialize, Deserialize};
	use influx_db_client::keys::Point;
	use influx_db_client::Value as Val;

	#[derive(Clone, Debug, Serialize, Deserialize)]
	#[serde(deny_unknown_fields)]
	pub struct Node {
		pub firstseen: String, //todo: use chrono
		pub lastseen: String,  //todo: use chrono
		pub flags: Flags,
		pub nodeinfo: Nodeinfo,
		pub statistics: Statistics,
	}

	#[derive(Clone, Debug, Serialize, Deserialize)]
	#[serde(deny_unknown_fields)]
	pub struct Statistics {
		pub clients: i64,
		pub loadavg: Option<f64>,
		pub memory_usage: Option<f64>,
		pub rootfs_usage: Option<f64>,
		pub traffic: Option<Traffic>,
		pub uptime: Option<f64>,
		pub vpn_peers: Option<Vec<String>>,
	}

	#[derive(Clone, Debug, Serialize, Deserialize)]
	#[serde(deny_unknown_fields)]
	pub struct Traffic {
		pub forward: TrafficIO,
		pub mgmt_rx: TrafficIO,
		pub mgmt_tx: TrafficIO,
		pub rx: TrafficIO,
		pub tx: TrafficIO,
	}

	#[derive(Clone, Debug, Serialize, Deserialize)]
	#[serde(deny_unknown_fields)]
	pub struct TrafficIO {
		pub bytes: i64,
		pub packets: i64,
		pub dropped: Option<i64>
	}

	#[derive(Clone, Debug, Serialize, Deserialize)]
	pub struct Flags {
		pub online: bool,
		pub uplink: bool,
	}

	#[derive(Clone, Debug, Serialize, Deserialize)]
	pub struct Nodeinfo {
		pub hardware: Hardware,
		pub hostname: String,
		pub location: Option<Location>,
		pub network: Network,
		pub node_id: String,
		pub owner: Option<Owner>,
		pub software: Software,
	}

	#[derive(Clone, Debug, Serialize, Deserialize)]
	pub struct Owner {
		pub contact: String,
	}

	// todo: use proper types for geodata
	#[derive(Clone, Debug, Serialize, Deserialize)]
	pub struct Location {
		pub altitude: Option<f64>,
		pub latitude: f64,
		pub longitude: f64,
	}

	#[derive(Clone, Debug, Serialize, Deserialize)]
	pub struct Hardware {
		pub model: Option<String>,
		pub nproc: Option<i64>,
	}

	// todo: use proper types
	#[derive(Clone, Debug, Serialize, Deserialize)]
	pub struct Network {
		pub addresses: Vec<String>, // todo: use proper types
		pub mac: String,
		pub mesh_interfaces: Option<Value>,
	}

	#[derive(Clone, Debug, Serialize, Deserialize)]
	pub struct Software {
		pub autoupdater: Option<Autoupdater>,
		#[serde(rename = "batman-adv")]
		pub batman_adv: BatmanAdv,
		pub fastd: Option<Fastd>,
		pub firmware: Firmware,
	}

	#[derive(Clone, Debug, Serialize, Deserialize)]
	pub struct Autoupdater {
		pub branch: String,
		pub enabled: bool
	}

	#[derive(Clone, Debug, Serialize, Deserialize)]
	pub struct Fastd {
		pub version: String,
		pub enabled: bool
	}

	#[derive(Clone, Debug, Serialize, Deserialize)]
	pub struct BatmanAdv {
		pub compat: Option<i64>,
		pub version: String
	}

	#[derive(Clone, Debug, Serialize, Deserialize)]
	pub struct Firmware {
		pub base: String,
		pub release: String
	}



	impl Node {
		pub fn get_measurement(&self, data: &mut Point) {

			data.add_tag("node_id", Val::String(self.nodeinfo.node_id.clone()));
			data.add_tag("firmware", Val::String(self.nodeinfo.software.firmware.base.clone()));
			data.add_tag("is_online", Val::Boolean(self.flags.online));
			data.add_tag("has_uplink", Val::Boolean(self.flags.uplink));


			data.add_field("node_id", Val::String(self.nodeinfo.node_id.clone()));

			data.add_field("online", Val::Boolean(self.flags.online));
			data.add_field("uplink", Val::Boolean(self.flags.uplink));
			data.add_field("firmware_base", Val::String(self.nodeinfo.software.firmware.base.clone()));
			data.add_field("firmware_release", Val::String(self.nodeinfo.software.firmware.release.clone()));

			data.add_field("clients", Val::Integer(self.statistics.clients));

			if let Some(model) = &self.nodeinfo.hardware.model {
				data.add_field("hardware_model", Val::String(model.clone()));
			}

			if let Some(nproc) = self.nodeinfo.hardware.nproc {
				data.add_field("hardware_nproc", Val::Integer(nproc));
			}

			if let Some(au) = &self.nodeinfo.software.autoupdater {
				data.add_field("software_autoupdate_enabled", Val::Boolean(au.enabled));
				data.add_field("software_autoupdate_branch", Val::String(au.branch.clone()));
			}

			if let Some(fastd) = &self.nodeinfo.software.fastd {
				data.add_field("software_fastd_enabled", Val::Boolean(fastd.enabled));
				data.add_field("software_autoupdate_branch", Val::String(fastd.version.clone()));
			}

			data.add_field("software_batman_version", Val::String(self.nodeinfo.software.batman_adv.version.clone()));
			if let Some(compat) = self.nodeinfo.software.batman_adv.compat {
				data.add_field("software_batman_compat", Val::Integer(compat));
			}


			if let Some(loadavg) = self.statistics.loadavg {
				data.add_field("statistics_loadavg", Val::Float(loadavg));
			}

			if let Some(mem) = self.statistics.memory_usage {
				data.add_field("statistic_memory_usage", Val::Float(mem));
			}

			if let Some(fs_use) = self.statistics.rootfs_usage {
				data.add_field("statistic_rootfs_usage", Val::Float(fs_use));
			}

			if let Some(uptime) = self.statistics.uptime {
				data.add_field("statistics_uptime", Val::Float(uptime));
			}

			if let Some(traffic) = &self.statistics.traffic {
				data.add_field("traffic_rx_bytes", Val::Integer(traffic.rx.bytes));
				data.add_field("traffic_tx_bytes", Val::Integer(traffic.tx.bytes));
				data.add_field("traffic_rx_packets", Val::Integer(traffic.rx.packets));
				data.add_field("traffic_tx_packets", Val::Integer(traffic.tx.packets));

				data.add_field("traffic_forwarded_bytes", Val::Integer(traffic.forward.bytes));
				data.add_field("traffic_forwarded_bytes", Val::Integer(traffic.forward.bytes));
				data.add_field("traffic_forwarded_packets", Val::Integer(traffic.forward.packets));
				data.add_field("traffic_forwarded_packets", Val::Integer(traffic.forward.packets));
			}
		}
	}
}
