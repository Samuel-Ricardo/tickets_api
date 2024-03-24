mod base;
mod config;
mod controller;
mod crypt;
mod ctx;
mod error;
mod log;
mod middleware;
mod model;
mod observability;
mod router;
mod rpc;
mod server;
mod service;
mod util;

/* -- #[cfg(test)] | Commented during early development -- */
pub mod _dev_utils;

pub use self::config::config;
pub use self::error::{Error, Result};

use server::startup;

#[tokio::main]
async fn main() -> Result<()> {
    startup().await
}
