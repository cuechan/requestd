use crate::config;
use crate::output;
use crate::Error;
use crate::TABLE;
use chrono;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use eui48::MacAddress;
use ffhl_multicast;
use influx_db_client as influxdb;
use influx_db_client::Point;
use influx_db_client::Value as Val;
use log::{debug, error, warn, info, trace};
use postgres;
use reqwest;
use serde_json as json;
use serde_json::Value;
use std::ops;
use std::thread;
use std::time::Duration;
use rusqlite as sqlite;
use sqlite::params;
use crate::model;
use ffhl_multicast::ResponddResponse;
use crate::DATABASE_PATH;


const TAGS: &[&str] = &[
	"nodeinfo.software.firmware.base",
	"nodeinfo.software.fastd.version",
	"nodeinfo.software.autoupdater.enabled",
	"nodeinfo.software.autoupdater.branch",
	"nodeinfo.node_id",
	"nodeinfo.hardware.model",
	"flags.uplink",
	"flags.online",
];

pub fn node_collector(config: &config::Config) -> Result<(), ()> {
	info!("getting nodes from respondd");

	let requester = ffhl_multicast::ResponderService::start(&config.respondd.iface, config.respondd.interval);
	let receiver = requester.get_receiver();


	thread::spawn(move || {
		loop {
			info!("request new data");
			requester.request(ffhl_multicast::RequestType::Statisitcs);
			requester.request(ffhl_multicast::RequestType::Nodeinfo);
			requester.request(ffhl_multicast::RequestType::Neighbors);
			thread::sleep(Duration::from_secs(30));
		}
	});


	let db = sqlite::Connection::open(DATABASE_PATH).unwrap();

	db.execute_batch(include_str!("./init_db.sql")).unwrap();

	for node_response in receiver {
		if !node_response.response.is_object() || node_response.response.as_object().unwrap().is_empty() {
			warn!("received empty object");
			continue;
		}

		save_raw_response(&db, node_response.clone()).unwrap();

		// let data = json::from_value::<model::Response>(node_response.response.clone()).unwrap();

		// match data {
		// 	model::Response::Nodeinfo(ni) => println!("{:#?}", ni.node_id),

		// 	_ => ()
		// }
	}















	// let flux = influxdb::Client::new(
	//     format!("http://{}:{}", config.db.influx, 8086),
	//     config.db.database.clone(),
	// );

	// let all_nodes: Vec<model::Node> = receiver.into_iter().map(|x| {
	// 	json::from_str(&x.response).unwrap()
	// }).collect();

	// let timez = Utc::now();
	// let total_nodes = all_nodes.len();
	// info!("got {} nodes", total_nodes);

	// let probe = Probe::new(all_nodes).get_some_stats();

	// info!("loadavg: {:#?}", probe);

	// let mut point = influxdb::Point::new("stats");
	// point.add_field("load", Val::Float(probe.0));
	// point.add_field("clients", Val::Integer(probe.1));
	// point.add_field("bytes_forwarded", Val::Integer(probe.2.forward.bytes));
	// point.add_field("bytes_mgmt_rx", Val::Integer(probe.2.mgmt_rx.bytes));
	// point.add_field("bytes_mgmt_tx", Val::Integer(probe.2.mgmt_tx.bytes));
	// point.add_field("nodes", Val::Integer(total_nodes as i64));

	// flux.write_point(point, Some(influxdb::Precision::Minutes), None).unwrap();

	Err(())
}




fn save_raw_response(db: &sqlite::Connection, response: ResponddResponse) -> Option<usize> {
	trace!("saving raw response");
	let changed = db.execute(
		"INSERT INTO raw_responses (timestamp, remote, response) VALUES (?, ?, ?)",
		&[Utc::now().timestamp().to_string(), response.remote.ip().to_string(), json::to_string(&response.response).unwrap()]
	).unwrap();

	Some(changed)
}

// pub struct Probe {
// 	nodes: Vec<model::Node>,
// }

// impl Probe {
// 	fn new(nodedata: Vec<model::Node>) -> Self {
// 		Self {
// 			nodes: nodedata
// 		}
// 	}

// 	pub fn get_some_stats(&self) -> (f64, i64, model::Traffic) {
// 		let mut clients = 0;
// 		let mut load_avg: f64 = 0.0;
// 		let mut load_avg_counter = 0;

// 		let mut traffic = model::Traffic::default();

// 		for node in self.nodes.iter() {
// 			match node.statistics.loadavg {
// 				None => continue,
// 				Some(x) => {
// 					load_avg_counter += 1;
// 					load_avg += x;
// 				}
// 			}

// 			match &node.statistics.traffic {
// 				None => continue,
// 				Some(x) => {
// 					traffic += x.clone();
// 				}
// 			}

// 			clients += node.statistics.clients;
// 		}

// 		(load_avg/load_avg_counter as f64, clients, traffic)
// 	}
// }

// pub fn store_node_influx(influx: &influxdb::Client, time: DateTime<Utc>, node: &serde_json::Value) -> Result<(), Error> {
// 	let mut measurement = influxdb::keys::Point::new("nodes");
// 	measurement.add_timestamp(time.timestamp_millis());

// 	flatten(
// 		String::new(),
// 		&node,
// 		&mut measurement
// 	);

// 	debug!("insert into influx");
// 	influx.write_point(
// 		measurement,
// 		Some(influxdb::keys::Precision::Milliseconds),
// 		None
// 	).map_err(Error::Influx)
// }

// pub fn store_node_postgres(psql: &postgres::Connection, time: DateTime<Utc>, node: &model::Node) -> Result<(), ()> {
// 	let check_existence = psql.prepare(&format!("
// 		SELECT count(*) AS count
// 		FROM {0}
// 		WHERE
// 			timestamp::timestamptz = date_trunc('minute', $1::timestamptz)
// 			AND
// 			(data->'nodeinfo'->>'node_id') = $2
// 		", TABLE
// 	)).unwrap();

// 	trace!("prepare sql statement");
// 	let insert_query = psql.prepare(&format!(
// 		"INSERT INTO {0}
// 		(timestamp, data) VALUES (date_trunc('minute', $1::timestamptz), $2)",
// 		TABLE
// 	)).unwrap();

// 	debug!("checking: {} {}", time, node.nodeinfo.node_id);
// 	let rows = check_existence.query(&[
// 			&time,
// 			&node.nodeinfo.node_id
// 		]).unwrap();

// 	if rows.get(0).get::<_, i64>(0) > 0 {
// 		debug!("skipping: {} {}", node.nodeinfo.node_id, time);
// 	} else {
// 		debug!("inserting: {} {}", time, node.nodeinfo.node_id);
// 		insert_query.execute(&[
// 			&time,
// 			&serde_json::to_value(&node).unwrap(),
// 		]).unwrap();
// 	}

// 	check_existence.finish().unwrap();
// 	insert_query.finish().unwrap();
// 	Ok(())
// }

// 	#[derive(Debug, Clone, Default)]
// 	pub struct Stats {
// 		pub clients: i64,

// 		// traffic
// 		pub total_rx_bytes: i64,
// 		pub total_tx_bytes: i64,
// 		pub total_rx_packets: i64,
// 		pub total_tx_packets: i64,
// 		pub total_forwarded: i64,
// 		pub rx_tx_delta: i64,

// 		pub nodes_online: i64,
// 		pub nodes_uplink: i64,
// 		pub nodes: i64,
// 	}

// 	impl Stats {
// 		pub fn new_empty() -> Self {
// 			Self::default()
// 		}

// 		pub fn get_measurement(&self, data: &mut Point) {
// 			data.add_field("clients", Val::Integer(self.clients));
// 			data.add_field("nodes.online", Val::Integer(self.nodes_online));
// 			data.add_field("nodes.uplink", Val::Integer(self.nodes_uplink));
// 			data.add_field("nodes", Val::Integer(self.nodes));
// 			data.add_field("traffic.forwarded", Val::Integer(self.total_forwarded));
// 			data.add_field("traffic.rx_tx_delta", Val::Integer(self.rx_tx_delta));
// 		}
// 	}
// }

// pub fn flatten(mut key: String, val: &serde_json::Value, point: &mut Point) {
// 	if !val.is_object() && !val.is_array() {
// 		trace!("{} => {:?}", key, val);
// 	}

// 	match val {
// 		Value::Bool(b) => {
// 			if is_tag(&key) {
// 				point.add_tag(&key, Val::Boolean(*b));
// 			}
// 			else {
// 				point.add_field(&key, Val::Boolean(*b));
// 			}
// 		},
// 		Value::Number(n) => {
// 			if is_tag(&key) {
// 				match (n.is_i64(), n.is_u64(), n.is_f64()) {
// 					(false, false, true ) => point.add_tag(&key, Val::Float(n.as_f64().unwrap())),
// 					(true , true , false) => point.add_tag(&key, Val::Integer(n.as_i64().unwrap())),
// 					(true , false, false) => point.add_tag(&key, Val::Integer(n.as_i64().unwrap())),
// 					(false, true , false) => point.add_tag(&key, Val::Integer(n.as_u64().unwrap() as i64)),
// 					_ => panic!("aahhhhhhh"),
// 				};
// 			} else {
// 				match (n.is_i64(), n.is_u64(), n.is_f64()) {
// 					(false, false, true ) => point.add_field(&key, Val::Float(n.as_f64().unwrap())),
// 					(true , true , false) => point.add_field(&key, Val::Integer(n.as_i64().unwrap())),
// 					(true , false, false) => point.add_field(&key, Val::Integer(n.as_i64().unwrap())),
// 					(false, true , false) => point.add_field(&key, Val::Integer(n.as_u64().unwrap() as i64)),
// 					_ => panic!("aahhhhhhh"),
// 				};
// 			}

// 		},
// 		Value::String(s) => {
// 			if is_tag(&&key) {
// 				point.add_tag(&key, Val::String(s.clone()));
// 			} else {
// 				point.add_field(&key, Val::String(s.clone()));
// 			}
// 		},
// 		Value::Object(o) => {
// 			for (nkey, nval) in o {
// 				match &key.is_empty() {
// 					true => flatten(nkey.clone(), nval, point),
// 					false => flatten(vec![key.clone(), nkey.clone()].join("."), nval, point),
// 				};
// 			}
// 		}
// 		_ => ()
// 	}
// }

// fn is_tag(key: &String) -> bool {
// 	for i in TAGS {
// 		if i == &key {
// 			return true
// 		}
// 	}

// 	false
// }

// some stat things

// stats.clients += node.statistics.clients;
// stats.nodes += 1;

// stats.nodes_online += if node.flags.online {1} else {0};
// stats.nodes_uplink += if node.flags.uplink {1} else {0};

// stats.total_forwarded += if let Some(traffic) = &node.statistics.traffic {traffic.forward.bytes} else {0};
// stats.rx_tx_delta += if let Some(traffic) = &node.statistics.traffic {traffic.rx.bytes - traffic.rx.bytes} else {0};
