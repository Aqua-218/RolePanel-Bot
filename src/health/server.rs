use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tokio::sync::watch;

use crate::gateway::event_loop::GatewayState;

pub async fn run_health_server(
    port: u16,
    pool: PgPool,
    gateway_state: watch::Receiver<GatewayState>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("Health server listening on {}", addr);

    let pool = Arc::new(pool);

    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => {
                tracing::info!("Shutting down health server");
                break;
            }
            result = listener.accept() => {
                let (stream, _) = result?;
                let io = TokioIo::new(stream);
                let pool = pool.clone();
                let gateway_state = gateway_state.clone();

                tokio::spawn(async move {
                    let service = service_fn(move |req| {
                        let pool = pool.clone();
                        let gateway_state = gateway_state.clone();
                        async move {
                            handle_request(req, pool, gateway_state).await
                        }
                    });

                    if let Err(e) = http1::Builder::new()
                        .serve_connection(io, service)
                        .await
                    {
                        tracing::error!("Error serving connection: {}", e);
                    }
                });
            }
        }
    }

    Ok(())
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    pool: Arc<PgPool>,
    gateway_state: watch::Receiver<GatewayState>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = match req.uri().path() {
        "/health/live" => {
            // Liveness: just check if we're running
            Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from("OK")))
                .unwrap()
        }
        "/health/ready" => {
            // Readiness: check gateway and database
            let gateway_ok = gateway_state.borrow().connected;
            let db_ok = check_database(&pool).await;

            if gateway_ok && db_ok {
                Response::builder()
                    .status(StatusCode::OK)
                    .body(Full::new(Bytes::from("OK")))
                    .unwrap()
            } else {
                let mut reasons = Vec::new();
                if !gateway_ok {
                    reasons.push("gateway disconnected");
                }
                if !db_ok {
                    reasons.push("database unreachable");
                }

                Response::builder()
                    .status(StatusCode::SERVICE_UNAVAILABLE)
                    .body(Full::new(Bytes::from(reasons.join(", "))))
                    .unwrap()
            }
        }
        "/health" => {
            // Combined health check
            let gateway_ok = gateway_state.borrow().connected;
            let db_ok = check_database(&pool).await;

            let status = if gateway_ok && db_ok {
                StatusCode::OK
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            };

            let body = serde_json::json!({
                "status": if status == StatusCode::OK { "healthy" } else { "unhealthy" },
                "gateway": gateway_ok,
                "database": db_ok,
            });

            Response::builder()
                .status(status)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(body.to_string())))
                .unwrap()
        }
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("Not Found")))
            .unwrap(),
    };

    Ok(response)
}

async fn check_database(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1").execute(pool).await.is_ok()
}
