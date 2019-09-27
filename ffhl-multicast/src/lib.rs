use std::net::{UdpSocket, Ipv6Addr, SocketAddr,SocketAddrV6};
use std::thread;
use std::time::Duration;
use libflate;
use libflate::deflate::Decoder;
use flate2::read::DeflateDecoder;
use std::io::Read;
use std::io::Cursor;
use log::{error, warn, info, debug, trace};
use libc;
use std::string::ToString;
use std::sync::{Mutex, Arc};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::io::ErrorKind;
use serde_json as json;
use serde_json::Value;


const CMD: &str = "GET nodeinfo";
const MLTCST_GROUP: &str = "ff02::2:1001";
const MLTCST_IFACE: u32 = 3;
const PORT: u16 = 16000;




/// The service object that can be used to
/// request data or stop the thread
pub struct ResponderService {
	interface: u32,
	rx: Receiver<ResponddResponse>,
	status: SharedReceiverLoopStatus,
	thread: thread::JoinHandle<()>,
}



struct ReceiverLoopStatus {
	interval: u64,
	running: bool,
	socket: UdpSocket,
}

type SharedReceiverLoopStatus = Arc<Mutex<ReceiverLoopStatus>>;



impl ResponderService {

	/// Request a specific response
	pub fn request(&self, what: RequestType) {
		let address = SocketAddrV6::new(
			MLTCST_GROUP.parse().unwrap(),
			1001,
			0,
			self.interface
		);

		trace!("requesting {} at {}", what.to_string(), address);

		self.status.lock().unwrap().socket.send_to(
			format!("GET {}", what.to_string()).as_bytes(),
			address
		).unwrap();


		trace!("request sent");
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
			let s = UdpSocket::bind("[::]:16000").unwrap();
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
		let handle = thread::spawn(move || {
			receiver_loop(stat, tx)
		});

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
			if e.kind() == ErrorKind::WouldBlock {
				trace!("no data");
			}
			else {
				warn!("unknown error occured");
			}

			thread::sleep(Duration::from_secs(5));
			continue
		};


		let (bytes_read, remote) = recv_result.unwrap();
		trace!("read {} bytes", bytes_read);
		let mut response = String::new();
		DeflateDecoder::new(&data[..bytes_read]).read_to_string(&mut response).unwrap();


		debug!("received data: {}", response);

		let json_ = match json::from_str(&response) {
			Err(e) => {
				error!("can't parse json {}", e);
				continue;
			},
			Ok(r) => r
		};


		let resp = ResponddResponse {
			remote: remote,
			response: json_
		};

		tx.send(resp).unwrap();
	}
}


#[derive(Debug, Clone)]
pub enum RequestType {
	Nodeinfo,
	Statisitcs,
	Neighbors
}


impl ToString for RequestType {
	fn to_string(&self) -> String {
		match self {
			Self::Nodeinfo => "nodeinfo".to_owned(),
			Self::Statisitcs => "statistics".to_owned(),
			Self::Neighbors => "neighbours".to_owned(),
		}
	}
}




#[derive(Debug, Clone)]
pub struct ResponddResponse {
	/// remote address
	pub remote: SocketAddr,
	/// the data
	pub response: Value,
}




pub fn if_to_index(interface: &str) -> Option<u32> {
	let i: u32 = unsafe {
		libc::if_nametoindex(interface.as_ptr() as *const i8).into()
	};

	trace!("iface index {:#?}", i);

	if i <= 0 {
		return None;
	}

	Some(i)
}
