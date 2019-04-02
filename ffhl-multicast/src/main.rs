use std::net::{UdpSocket, Ipv6Addr, SocketAddr};
use std::thread::{sleep_ms};

const command: &str = "GET statistics";


fn main() {
    let socket = UdpSocket::bind("[::]:1001").unwrap();
	let multicast_addr: Ipv6Addr = "ff02::2:1001".parse().unwrap();


	loop {
		socket.send_to(command.as_bytes(), SocketAddr::new(multicast_addr.into(), 1001)).unwrap();

		socket.join_multicast_v6(&multicast_addr, 0).unwrap();

		let mut data = [0; 4096];
		socket.recv_from(&mut data).unwrap();

		println!("data received");
		println!("{:#?}", String::from_utf8_lossy(&data));
	}
}
