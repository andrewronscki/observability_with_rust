use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};
use std::convert::Infallible;
use std::net::SocketAddr;

pub async fn serve_metrics(exporter: PrometheusExporter) {
    let addr = SocketAddr::from(([0, 0, 0, 0], 9464));

    let make_svc = make_service_fn(move |_| {
        let exporter = exporter.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |_req: Request<Body>| {
                let exporter = exporter.clone();
                async move {
                    let encoder = TextEncoder::new();
                    let metric_families = exporter.registry().gather();
                    let mut buffer = Vec::new();
                    encoder.encode(&metric_families, &mut buffer).unwrap();

                    Ok::<_, Infallible>(
                        Response::builder()
                            .status(200)
                            .header("Content-Type", encoder.format_type())
                            .body(Body::from(buffer))
                            .unwrap(),
                    )
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);
    println!("Servidor de métricas rodando em http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("Erro no servidor de métricas: {}", e);
    }
}
