use axum::{http::StatusCode, response::IntoResponse, routing::get, Json as ResponseJson, Router};
use log::info;
use open_telemetry::init_trace;
use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use tracing::{event, instrument, Level};
use tracing_subscriber::layer::SubscriberExt;

mod custom_error;
mod open_telemetry;

#[tokio::main]
#[instrument]
async fn main() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = init_trace().unwrap();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::Registry::default().with(telemetry);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Define a rota principal
    let app = Router::new().nest("/test", test_routes());

    // Inicia o servidor Axum
    let port = "3000";

    let listener = tokio::net::TcpListener::bind("0.0.0.0:".to_string() + &port)
        .await
        .unwrap();

    info!("🚀 App is running");
    axum::serve(listener, app).await.unwrap();
}

fn test_routes() -> Router {
    Router::new().route("/", get(test))
}

#[instrument]
async fn test() -> Result<impl IntoResponse, (StatusCode, custom_error::CustomError)> {
    event!(Level::INFO, "reason" = "Alguém acessou a rota de teste");
    Ok((StatusCode::OK, ResponseJson("test ok")))
}
