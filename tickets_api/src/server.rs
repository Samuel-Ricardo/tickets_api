use axum::{middleware, Router};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;

use crate::controller::ticket::TicketController;
use crate::error::Result;
use crate::middleware::auth::mw_ctx_resolver;
use crate::middleware::response::main_response_mapper;
use crate::router::{self, hello_router, login, tickets};

pub async fn startup() -> Result<()> {
    let controller = TicketController::new().await.unwrap();

    let routes: Router = Router::new()
        .merge(hello_router())
        .merge(login::routes())
        .nest("/api", tickets::routes(controller.clone()))
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            controller.clone(),
            mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(router::statics());

    let listener: TcpListener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind port 8080");

    println!("Server running on 0.0.0.0:8080");
    axum::serve(listener, routes.into_make_service())
        .await
        .unwrap();

    Ok(())
}
