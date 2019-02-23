use crate::config;
use reqwest;
use serde_json;
use postgres;
use chrono;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;


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

	let psql = postgres::Connection::connect(config.db.connection_params(), postgres::TlsMode::None).unwrap();

	let stmt = psql.prepare("
		INSERT INTO nodes
		(timestamp, nodedata)
		VALUES ($1, $2);
	").unwrap();


	for node in nodes.as_object().unwrap().get("nodes").unwrap().as_array().unwrap() {
		stmt.execute(&[&time_z, &node]).unwrap();
	}

	Err(())
}





// to lazy to do this. Just throw everything to postgres and let it handle it

pub mod model {
	use serde_json::Value;

	pub struct Node {
		firstseen: String, //todo: use chrono
		lastseen: String,  //todo: use chrono
		flags: Flags,
		nodeinfo: Nodeinfo,
	}

	pub struct Flags {
		online: bool,
		uplink: bool,
	}

	pub struct Nodeinfo {
		hardware: Hardware,
		hostname: String,
		location: Location,
		network: Network,
		node_id: String,
		owner: Owner,
		software: Software,
	}

	pub struct Owner {
		contact: String,
	}

	// todo: use proper types for geodata
	pub struct Location {
		altitude: String,
		latitude: String,
		longitude: String,
	}

	pub struct Hardware {
		model: String,
		nproc: u8,
	}

	// todo: use proper types
	pub struct Network {
		addresses: Vec<String>, // todo: use proper types
		mac: String,
		mesh: Value,
	}

	pub struct Software {
		autoupdater: Value,
	}
}
