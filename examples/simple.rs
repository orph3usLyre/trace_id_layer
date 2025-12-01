use axum::{Router, routing::get};
use http::StatusCode;
use tokio::signal;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    let mut router = Router::new()
        .route("/", get(index))
        .route("/health", get(healthcheck));
    router = trace_id_layer::add_trace_id_middleware(router);

    let server = axum::serve(listener, router).with_graceful_shutdown(shutdown_signal());
    tracing::info!("Running");
    let _ = tokio::join!(server, async {
        shutdown_signal().await;
    });

    Ok(())
}

pub async fn index() -> StatusCode {
    info!("Index!");
    StatusCode::OK
}

pub async fn healthcheck() -> StatusCode {
    info!("Healthcheck!");
    StatusCode::OK
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
