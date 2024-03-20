mod config;
mod controller;
mod ctx;
mod error;
mod log;
mod middleware;
mod model;
mod observability;
mod router;
mod server;

pub use self::config::config;
pub use self::error::{Error, Result};

use server::startup;

#[tokio::main]
async fn main() -> Result<()> {
    startup().await
}
