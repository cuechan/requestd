use paho_mqtt as mqtt;
use crate::Endpoint;
use crate::Collector;
use crate::CONFIG;
use crossbeam::channel as crossbeam;
use crate::NodeResponse;
use std::thread;
use std::time::Duration;
use serde_json as json;
use log::{error, warn, info, trace};
use std::sync::{Arc, Mutex};


const MQTT_QOS: i32 = 1;
const RECONNECT_WAIT: u64 = 1000;

pub struct Mqtt {
	mqtt_client: mqtt::client::Client,
	events_receiver: crossbeam::Receiver<NodeResponse>,
}


impl Mqtt {
	fn try_connect(&self) -> bool {
		self.mqtt_client.connect(
			mqtt::ConnectOptionsBuilder::new()
				// .keep_alive_interval(Duration::from_secs(20))
				.mqtt_version(mqtt::types::MQTT_VERSION_DEFAULT)
				.clean_session(true)
				.finalize(),
		).is_ok()
	}

	fn flush_receiver(&self) {
		while !self.events_receiver.is_empty() {
			self.events_receiver.recv();
		}
	}
}


impl Endpoint for Mqtt {
	fn new(c: Arc<Mutex<Collector>>) -> Self {
		let client = mqtt::Client::new(
			mqtt::CreateOptionsBuilder::new()
				.server_uri(CONFIG.mqtt.clone().unwrap().broker)
				// .mqtt_version(5)
				// .client_id(MQTT_CLIENT)
				.finalize(),
		)
		.expect("error creating mqtt client");

		Self {
			mqtt_client: client,
			events_receiver: c.lock().unwrap().get_events_receiver()
		}
	}

	fn start(self) -> ! {
		// self.mqtt_client.is_connected();

		for event in &self.events_receiver {
			info!("send mqtt event");
			let msg = mqtt::Message::new(CONFIG.mqtt.clone().unwrap().topic, json::to_string(&event).unwrap(), MQTT_QOS);
			trace!("sending mqtt message");


			match self.mqtt_client.publish(msg) {
				Err(mqtt::errors::Error::PahoDescr(c, _)) if c == -3 && !self.mqtt_client.is_connected() => {
					while !self.try_connect() {
						// remove all waiting events to prevent them form piling
						// up the channel and occupying memory
						warn!("mqqt broker disconnected. Trying to reconnect to mqtt broker");
						self.flush_receiver();
						thread::sleep(Duration::from_millis(RECONNECT_WAIT))
					}
				},
				Err(e) => {
					error!("cannot handle mqtt error: {}", e);
					panic!("mqtt error");
				},
				Ok(_) => (),
			}
		}

		panic!("event loop stopped");
	}
}

// fn build_client() -> mqtt::client::Client {
	// Connect and wait for it to complete or fail.
// 	client
// 		.connect(
// 			mqtt::ConnectOptionsBuilder::new()
// 				// .keep_alive_interval(Duration::from_secs(20))
// 				.mqtt_version(mqtt::types::MQTT_VERSION_DEFAULT)
// 				.clean_session(true)
// 				.finalize(),
// 		)
// 		.expect("Unable to connect");

// 	client
// }
