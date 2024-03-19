use server::startup;

mod controller;
mod ctx;
mod error;
mod log;
mod middleware;
mod model;
mod router;
mod server;

use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    startup().await
}
