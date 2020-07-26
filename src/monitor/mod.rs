use futures::executor::block_on;
use hyper::rt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use log::info;
use prometheus::{Encoder, TextEncoder};
use std::net::IpAddr;
use std::net::SocketAddr;
use tokio;
use tokio::prelude::*;
use tokio::runtime::Runtime;
use prometheus;

pub mod metrics;

const ADDRESS: &str = "0.0.0.0";
const PORT: u16 = 9092;


// mostly copied from example
// https://gist.github.com/breeswish/bb10bccd13a7fe332ef534ff0306ceb5

async fn metric_service(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
	let encoder = TextEncoder::new();
	let mut buffer = vec![];
	let mf = prometheus::gather();
	encoder.encode(&mf, &mut buffer).unwrap();
	Ok(Response::builder()
		.header(hyper::header::CONTENT_TYPE, encoder.format_type())
		.body(Body::from(buffer))
		.unwrap())
}



pub fn start_exporter() {
	let addr: SocketAddr = (ADDRESS.parse::<IpAddr>().unwrap(), PORT).into();
	let service = make_service_fn(|_| async {
		Ok::<_, hyper::Error>(service_fn(|_req| metric_service(_req)))
	});


	std::thread::spawn(move || {
		let mut rt = Runtime::new().unwrap();
		rt.block_on(async {
			let server = Server::bind(&addr).serve(service);
			tokio::spawn(server).await.unwrap().unwrap();
		});

		info!("runtime is done");
	});
}
