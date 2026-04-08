mod routes;
mod state;

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use gravitas_fetch::mock::MockSource;
use gravitas_fetch::tradier::TradierSource;

use state::AppState;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    tracing_subscriber::fmt::init();

    let source: Box<dyn gravitas_fetch::source::DataSource> =
        if let Some(token) = secrets.get("TRADIER_TOKEN") {
            tracing::info!("Using Tradier data source");
            Box::new(TradierSource::new(token, false))
        } else {
            tracing::warn!("No TRADIER_TOKEN secret, using mock data source");
            Box::new(MockSource::new())
        };

    let state = AppState::new(source);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([axum::http::Method::GET])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let router = Router::new()
        .route("/health", get(routes::health::health))
        .route("/options/{symbol}", get(routes::options::get_options))
        .route("/gex/{symbol}", get(routes::gex::get_gex))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    Ok(router.into())
}
