use prometheus::{self, register_int_counter, IntCounter};
use lazy_static::lazy_static;


lazy_static! {
	pub static ref HIGH_FIVE_COUNTER: IntCounter =
		register_int_counter!("highfives", "Number of high fives received").unwrap();
}
