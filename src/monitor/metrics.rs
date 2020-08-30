use lazy_static::lazy_static;
use prometheus::{self, register_int_counter, register_int_gauge, IntCounter, IntGauge};

lazy_static! {
	pub static ref TOTAL_REQUESTS: IntCounter = register_int_counter!("total_requests", "Total number of sent requests").unwrap();
	pub static ref TOTAL_RESPONSES: IntCounter = register_int_counter!("total_responses", "Total number of received responses").unwrap();
	pub static ref HOOKS_WAITING: IntGauge = register_int_gauge!("hooks_waiting", "number of hooks waiting in queue").unwrap();
}
