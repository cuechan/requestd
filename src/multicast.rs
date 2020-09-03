use crate::metrics;
use crate::Timestamp;
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

/// wrapper for the socket.
///
/// Wrapped so we can use it on different threads
type SharedSocket = Arc<Mutex<Socket>>;

/// The service object that can be used to
/// request data or stop the thread
#[derive(Debug, Clone)]
pub struct RequesterService {
	interface: u32,
	rx: Receiver<ResponddResponse>,
	socket: SharedSocket,
	// thread: thread::JoinHandle<()>,
}

impl RequesterService {
	/// starts the respondd requester
	/// this is non-blocking and spawns it's own thread
	pub fn new(iface: &str) -> Self {
		let iface_n = if_to_index(&iface).expect(&format!("no such interface: {}", iface));

		let (tx, rx) = unbounded::<ResponddResponse>();
		let socket = Arc::new(Mutex::new({
			let s: Socket = Socket::new(Domain::ipv6(), Type::dgram(), Some(Protocol::udp())).unwrap();
			s.set_nonblocking(true).unwrap();
			s.bind(&SockAddr::from("[::]:16000".parse::<SocketAddrV6>().unwrap()))
				.unwrap();
			// s.set_ttl(1).unwrap();
			s
		}));

		trace!("starting multicast service: scopeid={}", iface_n);
		let socket_copy = socket.clone();
		thread::spawn(move || receiver_loop(socket_copy, tx));

		RequesterService {
			interface: iface_n,
			rx: rx,
			socket: socket,
			// thread: handle,
		}
	}

	/// Request a specific response
	pub fn request(&self, dst: &String, what: &Vec<String>) {
		let dest = SocketAddrV6::new(dst.parse().unwrap(), 1001, 0, self.interface);

		trace!("requesting {:?}", what);

		let ref socket = self.socket.lock().unwrap();

		metrics::TOTAL_REQUESTS.inc();

		if let Err(e) = socket.send_to(format!("GET {}", what.join(" ")).as_bytes(), &SockAddr::from(dest)) {
			error!("can't send multicast data: {}", e);
			info!("is there a route configured? see https://github.com/nodejs/help/issues/2073#issuecomment-533834373");
		}
	}

	/// get the a receiver where all parsed messages will pop out
	pub fn get_receiver(&self) -> Receiver<ResponddResponse> {
		self.rx.clone()
	}

	/// get a channel where you can send request
	pub fn get_requester(&self) -> Receiver<ResponddResponse> {
		self.rx.clone()
	}

	pub fn stop(self) {
		// self.status.lock().unwrap().running = false;
		// self.thread.join().unwrap();
	}
}

/// request data from respondd
fn receiver_loop(socket: SharedSocket, tx: Sender<ResponddResponse>) {
	loop {
		let mut data = [0; 65536];
		let recv_result;

		// use extra scope so the lock gets dropped after we are done receiving
		{
			let socket = socket.lock().unwrap();
			recv_result = socket.recv_from(&mut data);
		}

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

		let json_: Value = match json::from_str(&response) {
			Err(e) => {
				error!("can't parse json {}", e);
				continue;
			}
			Ok(r) => r,
		};

		if !json_.is_object() {
			warn!("received weird response: not a json object");
			continue;
		}

		let resp = ResponddResponse {
			timestamp: Utc::now(),
			remote: remote.as_std().expect("cant convert to `socket2::SockAddr`"),
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
