use crate::Timestamp;
use crate::CONFIG;
use chrono::Utc;
use crossbeam_channel::{unbounded, Receiver, Sender};
use flate2::read::DeflateDecoder;
use libc;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use serde_json as json;
use serde_json::Value;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::io::ErrorKind;
use std::io::Read;
use std::net::{SocketAddr, SocketAddrV6};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::metrics;

/// The service object that can be used to
/// request data or stop the thread
pub struct ResponderService {
	interface: u32,
	rx: Receiver<ResponddResponse>,
	status: SharedReceiverLoopStatus,
	thread: thread::JoinHandle<()>,
}

struct ReceiverLoopStatus {
	#[allow(dead_code)]
	interval: u64,
	running: bool,
	socket: Socket,
}

type SharedReceiverLoopStatus = Arc<Mutex<ReceiverLoopStatus>>;

impl ResponderService {
	/// Request a specific response
	pub fn request(&self, what: &Vec<String>) {
		let address = SocketAddrV6::new(
			CONFIG.respondd.multicast_address.parse().unwrap(),
			1001,
			0,
			self.interface,
		);

		trace!("requesting {:?} at {}", what, address);

		let ref socket = self.status.lock().unwrap().socket;

		metrics::TOTAL_REQUESTS.inc();

		socket.send_to(format!("GET {}", what.join(" ")).as_bytes(), &SockAddr::from(address))
			.expect("can't send data");
	}

	/// get the a receiver where all parsed messages will pop out
	pub fn get_receiver(&self) -> Receiver<ResponddResponse> {
		self.rx.clone()
	}

	/// starts the respondd requester
	/// this is non-blocking and spawn it's own thread
	pub fn start(iface: &str, interval: u64) -> Self {
		let iface_n = if_to_index(&iface).expect(&format!("no such interface: {}", iface));

		let (tx, rx) = unbounded::<ResponddResponse>();
		let socket = {
			// let s = Socket::new("[::]:16000").unwrap();
			let s = Socket::new(Domain::ipv6(), Type::dgram(), Some(Protocol::udp())).unwrap();
			// s.set_nonblocking(true).unwrap();
			s.set_nonblocking(true).unwrap();
			// s.set_ttl(1).unwrap();
			s
		};

		let status = Arc::new(Mutex::new(ReceiverLoopStatus {
			interval: interval,
			running: true,
			socket: socket,
		}));

		let stat = status.clone();
		let handle = thread::spawn(move || receiver_loop(stat, tx));

		trace!("starting multicast service: scopeid={}", iface_n);

		Self {
			interface: iface_n,
			rx: rx,
			status: status,
			thread: handle,
		}
	}

	pub fn stop(self) {
		self.status.lock().unwrap().running = false;
		self.thread.join().unwrap();
	}
}

/// request data from respondd
fn receiver_loop(shared_status: SharedReceiverLoopStatus, tx: Sender<ResponddResponse>) {
	loop {
		let status = shared_status.lock().unwrap();

		if !status.running {
			// stop loop
			drop(status);
			return;
		}

		let mut data = [0; 65536];
		let recv_result = status.socket.recv_from(&mut data);

		// drop ownership to free mutex
		drop(status);

		if let Err(ref e) = recv_result {
			if e.kind() != ErrorKind::WouldBlock {
				warn!("unknown error occured");
				error!("{:#?}", e.kind());
			}

			thread::sleep(Duration::from_secs(5));
			continue;
		}

		metrics::TOTAL_RESPONSES.inc();

		let (bytes_read, remote) = recv_result.unwrap();
		let mut response = String::new();
		DeflateDecoder::new(&data[..bytes_read])
			.read_to_string(&mut response)
			.unwrap();

		// trace!("received data: {}", response);

		let json_: Value = match json::from_str(&response) {
			Err(e) => {
				error!("can't parse json {}", e);
				continue;
			}
			Ok(r) => r,
		};

		// trace!("record: \n{:#?}", json_);

		// let mut record;
		if !json_.is_object() {
			warn!("received incompatible data: not a json object");
			continue;
		}

		let resp = ResponddResponse {
			timestamp: Utc::now(),
			remote: remote.as_std().expect("cant convert to std AockAddr"),
			response: json_,
		};

		tx.send(resp).unwrap();
	}
}

#[derive(Clone, Debug)]
pub enum Error {}

#[derive(Debug, Clone)]
pub struct ResponddResponse {
	pub timestamp: Timestamp,
	/// remote address
	pub remote: SocketAddr,
	/// the data
	pub response: Value,
}

pub fn if_to_index(interface: &str) -> Option<u32> {
	let i: u32 = unsafe { libc::if_nametoindex(interface.as_ptr() as *const i8).into() };

	trace!("iface index {:#?}", i);

	if i <= 0 {
		return None;
	}

	Some(i)
}
