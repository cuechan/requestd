mod endpoints;

use crate::collector::{Collector, Event};
use crate::nodedb::{self, NodeDb, NodeStatus};
use crate::CONFIG;
use actix_web::{
	dev::Server, http::StatusCode, middleware, rt, web, App, HttpRequest, HttpResponse, HttpServer, Result as WebResult,
};
use chrono::{DateTime, Utc};
use handlebars::{self, Handlebars};
use log::error;
use rocket;
use rocket::config::{Config, Environment};
use rocket::http::Status;
use rocket::response::content;
use rocket::response::content::Html;
use rocket::State;
use rocket::{get, post, routes};
use serde_json as json;
use serde_json::json;
use std;
use std::net::SocketAddr;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use tera;

const TEMPLATES: &[(&str, &str)] = &[
	("index", include_str!("../../templates/index.html")),
	("nodelist", include_str!("../../templates/nodelist.html",)),
	("head", include_str!("../../templates/head.html",)),
	("node", include_str!("../../templates/node.html",)),
	("navbar", include_str!("../../templates/navbar.html",)),
];

struct InternalState {
	hbs: tera::Tera,
	db: NodeDb,
	collector: Collector,
}

type AppState = Mutex<InternalState>;

#[get("/nodes")]
fn list_nodes(state: State<'_, AppState>) -> Html<String> {
	let mut state_ = state.lock().unwrap();
	// let nodes: Vec<String> = state_.db.get_all_nodes().iter().map(|n| format!("{:#?}", n)).collect();
	let nodes = state_.db.get_all_nodes().clone();

	let nodes_: Vec<json::Value> = nodes
		.iter()
		.map(|n| {
			json!({
				"last_seen_secs": Utc::now().signed_duration_since(n.last_seen).num_seconds(),
				"last_response": n.last_response,
				"nodeid": n.nodeid,
				"last_address": n.last_address,
				"status": n.status,
				"raw": n,
			})
		})
		.collect();

	// println!("{:#?}", nodes);
	let data = json!({ "nodes": nodes_ });
	let html = state_
		.hbs
		.render("nodelist", &tera::Context::from_serialize(&data).unwrap())
		.unwrap();

	Html(html)
}

#[get("/node/<nodeid>")]
fn node_details(state: State<'_, AppState>, nodeid: String) -> Html<String> {
	let mut state_ = state.lock().unwrap();
	// let nodes: Vec<String> = state_.db.get_all_nodes().iter().map(|n| format!("{:#?}", n)).collect();
	let node = state_.db.get_node(&nodeid).unwrap();

	let data = json!({
		"node": node,
		"nodeid": node.nodeid,
		"last_response": serde_json::to_string_pretty(&node.last_response).unwrap(),
		"last_response_secs": Utc::now().signed_duration_since(node.last_seen).num_seconds(),
		"status": node.status,
	});

	let html = state_
		.hbs
		.render("node", &tera::Context::from_serialize(&data).unwrap())
		.unwrap();

	Html(html)
}

#[get("/hook/<hook>")]
fn hooks_endpoint(state: State<'_, AppState>, hook: String) -> Result<Vec<u8>, Status> {
	let mut state_ = state.lock().unwrap();
	let nodes = state_.db.get_all_nodes().clone();
	// drop state_ early so we dont block other
	// request with long running hooks
	drop(state_);

	match endpoints::process_request(hook, json::to_value(&nodes).unwrap()) {
		Ok(r) => Ok(r),
		Err(e) => {
			// todo: include errors in response
			error!("runnig hook failed: {:#?}", e);
			Err(Status::InternalServerError)
		}
	}
}

#[get("/")]
fn index(state: State<'_, AppState>) -> Result<Html<String>, tera::Error> {
	let state_ = state.lock().unwrap();
	let nodes_online = state_
		.db
		.get_all_nodes()
		.iter()
		.filter(|n| n.status == NodeStatus::Up)
		.count();

	let nodes_offline = state_
		.db
		.get_all_nodes()
		.iter()
		.filter(|n| n.status == NodeStatus::Down)
		.count();

	let mut events = state_.collector.get_event_history().clone();
	events.reverse();

	let data = json!({
		"nodes_online": nodes_online,
		"responses_received": state_.collector.get_num_received(),
		"nodes_offline": nodes_offline,
		"recent_events": events,
	});

	let html = state_.hbs
		.render("index", &tera::Context::from_serialize(&data).unwrap())?;

	Ok(Html(html))
}

// fn run_app(state: AppState) -> std::io::Result<()> {
// 	let mut sys = rt::System::new("test");

// 	let state = web::Data::new(Arc::new(Mutex::new(state)));

// 	// srv is server controller type, `dev::Server`
// 	let srv = HttpServer::new(move || {
// 		App::new()
// 			// enable logger
// 			.wrap(middleware::Logger::default())
// 			.app_data(state.clone())
// 			.service(web::resource("/nodes").to(list_nodes))
// 			// .service(web::resource("/index.html").to(|| async { "Hello world!" }))
// 	})
// 	.workers(1)
// 	.bind("127.0.0.1:8000")?
// 	.run();

// 	sys.block_on(srv)
// }

pub fn main(db: nodedb::NodeDb, collector: Collector) {
	let templates = load_templates_tera();
	let appstate = InternalState {
		hbs: templates,
		db: db,
		collector: collector,
	};

	let config = rocket_config();

	rocket::custom(config)
		.mount("/", routes![list_nodes, node_details, index, hooks_endpoint])
		.manage(Mutex::new(appstate))
		.launch();
}

fn load_templates_tera() -> tera::Tera {
	let mut t = tera::Tera::default();

	for (name, template) in TEMPLATES {
		if let Err(e) = t.add_raw_template(name, template) {
			error!("failed to load template: {}", name);
			match &e.kind {
				tera::ErrorKind::Msg(m) => error!("{}", m),
				_ => error!("unknown error"),
			}
			error!("loading templates failed: {:#?}", e);
			std::process::exit(1);
		}
	}

	t
}

#[cfg(debug_assertions)]
fn rocket_config() -> Config {
	Config::build(Environment::Development).finalize().unwrap()
}

#[cfg(not(debug_assertions))]
fn rocket_config() -> Config {
	let listen: SocketAddr = CONFIG.web_listen.parse().unwrap();
	Config::build(Environment::Production)
		.address(listen.ip().to_string())
		.port(listen.port())
		.finalize()
		.unwrap()
}
