use axum::{middleware, Router};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::info;

use crate::controller::ticket::TicketController;
use crate::error::Result;
use crate::middleware::auth::mw_ctx_resolver;
use crate::middleware::response::main_response_mapper;
use crate::model::ModelManager;
use crate::router::{self, hello_router, login, tickets};
use crate::{_dev_utils, observability};

pub async fn startup() -> Result<()> {
    let _ = observability::startup();
    let _ = _dev_utils::init_dev();
    let manager = ModelManager::new().await.unwrap();

    let controller = TicketController::new().await.unwrap();

    let routes: Router = Router::new()
        .merge(hello_router())
        .merge(login::routes(manager.clone()))
        .nest("/api", tickets::routes(controller.clone()))
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            manager.clone(),
            mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(router::statics());

    let listener: TcpListener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind port 8080");

    info!("Server running on 0.0.0.0:8080");
    axum::serve(listener, routes.into_make_service())
        .await
        .unwrap();

    Ok(())
}
