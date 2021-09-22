use crate::collector::Collector;
#[allow(unused_imports)]
use log::{error, warn};
use rocket;
use rocket::{get, routes};
use rocket::config::{Config, Environment};
use rocket::response::content::Html;
use rocket::State;
use rocket_contrib::json::Json;
use std;
use std::sync::{Arc, Mutex};
use std::process;
use crate::NodeResponse;

const DATETIME_FORMAT: &str = "%F %T";


pub struct AppState {
	collector: Arc<Mutex<Collector>>,
}


#[get("/")]
fn index() -> Html<String> {
	Html(include_str!("./index.html").to_string())
}


#[get("/responses")]
fn all_responses(state: State<'_, AppState>) -> Json<Vec<NodeResponse>> {
	let collector = state.collector.lock().unwrap();
	let nodes = collector.all_responses();

	Json(nodes)
}


pub fn main(collector: Arc<Mutex<Collector>>) {

	let appstate = AppState {
		collector: collector,
	};

	let config = rocket_config();

	let status = rocket::custom(config)
		.mount("/", routes![
			all_responses,
			index,
		])
		.manage(appstate)
		.launch();

	error!("can't launch rocket");
	error!("{}", status);
	process::exit(1);
}


#[cfg(debug_assertions)]
fn rocket_config() -> Config {
	Config::build(Environment::Development).finalize().unwrap()
}

#[cfg(not(debug_assertions))]
fn rocket_config() -> Config {
	use std::net::SocketAddr;
	use crate::CONFIG;

	let listen: SocketAddr = CONFIG.web.clone().unwrap().listen;
	Config::build(Environment::Production)
		.address(listen.ip().to_string())
		.port(listen.port())
		.finalize()
		.unwrap()
}
