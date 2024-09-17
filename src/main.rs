use axum::{http::StatusCode, response::IntoResponse, routing::get, Json as ResponseJson, Router};
use log::info;
use open_telemetry::init_trace;
use opentelemetry::sdk::export::metrics::aggregation;
use opentelemetry::sdk::metrics::{processors, selectors};
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::{
    global,
    sdk::{metrics, Resource},
};
use opentelemetry::{Context, KeyValue};
use opentelemetry_prometheus;
use prometheus::serve_metrics;
use tracing::{event, instrument, Level};
use tracing_subscriber::layer::SubscriberExt;

mod custom_error;
mod open_telemetry;
mod prometheus;

#[tokio::main]
#[instrument]
async fn main() {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = init_trace().unwrap();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::Registry::default().with(telemetry);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Configura o recurso (resource) com o nome do serviço
    let resource = Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "observability_with_rust",
    )]);

    let controller = metrics::controllers::basic(processors::factory(
        selectors::simple::histogram(vec![]),
        aggregation::cumulative_temporality_selector(),
    ))
    .with_resource(resource)
    .build();

    // Inicializa o exporter do Prometheus com o controlador
    let exporter = opentelemetry_prometheus::exporter(controller.clone()).init();

    // Define o MeterProvider global
    global::set_meter_provider(controller);

    // Inicia o servidor de métricas em uma tarefa separada
    let exporter_clone = exporter.clone();
    tokio::spawn(async move {
        serve_metrics(exporter_clone).await;
    });

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

    // Obtém o meter global
    let meter = global::meter("observability_with_rust");

    // Cria um contador de requisições
    let request_counter = meter
        .u64_counter("app.request.count")
        .with_description("Contador de requisições")
        .init();

    // Cria o contexto atual
    let cx = Context::current();

    // Incrementa o contador com o contexto
    request_counter.add(&cx, 1, &[]);

    Ok((StatusCode::OK, ResponseJson("test ok")))
}
