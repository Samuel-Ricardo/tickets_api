use axum::Router;
use tokio::net::TcpListener;

use crate::router::hello_router;

#[tokio::main]
pub async fn startup() -> Result<()> {
    const routes: Router = Router::new().merge(hello_router());

    const listener: TcpListener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind port 8080");

    println!("Server running on 127.0.0.1:8080");
    axum::serve(listener, routes.into_make_service())
        .await
        .unwrap();

    Ok(())
}
