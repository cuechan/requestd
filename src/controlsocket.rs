use crate::nodedb::NodeDb;
use log::*;
use serde_json as json;
use std::fs;
#[allow(unused_imports)]
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net;
use std::path::Path;

/// at the moment we just dump the whole database
/// Someday we accept commadns to manually request data
pub fn start(mut db: NodeDb, address: &String) {
	let path = Path::new(&address);
	if path.exists() {
		fs::remove_file(path).expect("can't remove old socket");
	}

	let listener = net::UnixListener::bind(address).expect("can't bind to unixsocket");
	debug!(
		"bind to socket: {:#?}",
		listener.local_addr().unwrap().as_pathname().unwrap()
	);

	let f = unsafe { fs::File::from_raw_fd(listener.as_raw_fd()) };
	let mut p = f.metadata().unwrap().permissions();
	p.set_mode(0o664);
	f.set_permissions(p).unwrap();

	while let Ok((stream, addr)) = listener.accept() {
		info!("a new connection from {:?}", addr);
		let all_nodes = db.get_all_nodes();

		if let Err(e) = json::to_writer(stream, &all_nodes) {
			error!("error writing stream: {}", e);
			info!("you maybe want to check your script");
		}

		info!("bye!");
	}
}
