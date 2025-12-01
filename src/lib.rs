use std::time::Duration;

use axum::response::Response;
use http::{HeaderMap, Request};
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{Span, info};
use uuid::Uuid;

const TRACE_ID_HEADER: &str = "x-trace-id";

pub fn add_trace_id_middleware(router: axum::Router) -> axum::Router {
    router.layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<axum::body::Body>| {
                let span = tracing::info_span!("http-request", trace_id = tracing::field::Empty);
                // Extract or generate trace-id
                if let Some(trace_id) = request
                    .headers()
                    .get(TRACE_ID_HEADER)
                    .and_then(|v| v.to_str().ok())
                {
                    info!("Received request with trace_id: '{trace_id}'");
                    span.record("trace_id", trace_id);
                } else {
                    let trace_id = Uuid::now_v7();
                    info!("Received request without trace_id. Assigning: '{trace_id}'");
                    span.record("trace_id", trace_id.to_string());
                }
                span
            })
            .on_response(
                |_response: &Response<axum::body::Body>, latency: Duration, _span: &Span| {
                    tracing::debug!("response generated in '{latency:?}'")
                },
            )
            .on_eos(
                |_trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span| {
                    tracing::debug!("stream closed after '{stream_duration:?}'")
                },
            )
            .on_failure(
                |error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                    tracing::warn!(
                        "something went wrong. Error data: '{error:?}'. Latency: '{latency:?}'"
                    )
                },
            ),
    )
}
