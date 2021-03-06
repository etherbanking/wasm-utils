extern crate parity_wasm;
extern crate env_logger;
extern crate byteorder;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;

pub mod rules;

mod optimizer;
mod gas;
mod symbols;
mod logger;
mod ext;
mod pack;
mod nondeterminism_check;
mod runtime_type;

pub use optimizer::{optimize, Error as OptimizerError};
pub use gas::inject_gas_counter;
pub use logger::init_log;
pub use ext::externalize;
pub use pack::pack_instance;
pub use nondeterminism_check::is_deterministic;
pub use runtime_type::inject_runtime_type;
