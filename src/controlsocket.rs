use std::os::unix::net;
use std::path::Path;
use log::*;
use std::io::Write;
use std::fs;
use crate::nodedb::NodeDb;
use serde_json as json;





/// at the moment we just dump the whole database
/// Someday we accept commadns to manually request data
pub fn start(mut db: NodeDb, address: &String) {
	let path = Path::new(&address);
	if path.exists() {
		fs::remove_file(path).expect("can't remove old socket");
	}


	let listener = net::UnixListener::bind(address).expect("can't bind to unixsocket");

	while let Ok((stream, addr)) = listener.accept() {
		info!("a new connection from {:?}", addr);
		let all_nodes = db.get_all_nodes();

		json::to_writer(stream, &all_nodes).expect("error writing stream");

		info!("bye!");
	}
}
