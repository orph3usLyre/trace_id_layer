use std::{fmt::Display, ops::Deref, time::Duration};

use axum::{
    extract::FromRequestParts,
    middleware::{self, Next},
    response::Response,
};
use http::{HeaderMap, Request, StatusCode, request::Parts};
use tower::ServiceBuilder;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{Span, error, info};
use uuid::Uuid;

const TRACE_ID_HEADER: &str = "x-trace-id";

/// The trace ID extracted or generated for this request.
///
/// Use this as an extractor in your handlers to access the trace ID:
///
/// ```rust
/// use trace_id_layer::TraceId;
///
/// async fn my_handler(trace_id: TraceId) -> String {
///     format!("Request trace ID: {}", trace_id)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TraceId(Uuid);

impl Deref for TraceId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TraceId {
    pub fn uuid(&self) -> Uuid {
        self.0
    }
}

impl<S> FromRequestParts<S> for TraceId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        const ERR_MSG: &str = "TraceId extension missing. Did you apply add_trace_id_middleware?";
        parts
            .extensions
            .get::<TraceId>()
            .cloned()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, ERR_MSG))
            .inspect_err(|e| error!("{ERR_MSG}. Error: '{e:?}'"))
    }
}

/// Middleware to inject trace_id into request extensions
async fn inject_trace_id(mut request: Request<axum::body::Body>, next: Next) -> Response {
    // Extract or generate trace-id
    let trace_id = if let Some(trace_id) =
        request.headers().get(TRACE_ID_HEADER).and_then(|v| {
            v.to_str()
                .inspect_err(|e| error!("Unable to convert trace-id header to string: '{e:?}'"))
                .ok()
                .and_then(|trace_id| Uuid::parse_str(trace_id).inspect_err(|e| error!("Unable to parce trace-id header to Uuid. Received: '{trace_id}'. Error: '{e:?}'")).ok())
        }) {
        trace_id
    } else {
        Uuid::now_v7()
    };

    // Store in request extensions for handler access
    request.extensions_mut().insert(TraceId(trace_id));

    next.run(request).await
}

pub fn add_trace_id_middleware(router: axum::Router) -> axum::Router {
    router
        // NOTE: It's required to use ServiceBuilder (rather than chain `.layer()` on router),
        // since otherwise `TraceId` isn't exposed in the extensions
        .layer(ServiceBuilder::new()
            // inject trace_id into request extensions
            .layer(middleware::from_fn(inject_trace_id))
            // then add tracing layer
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request<axum::body::Body>| {
                        let span =
                            tracing::info_span!("http-request", trace_id = tracing::field::Empty);

                        // Get trace_id from extensions (already injected by previous middleware)
                        if let Some(trace_id) = request.extensions().get::<TraceId>() {
                            // Check if it came from header or was generated
                            if request.headers().get(TRACE_ID_HEADER).is_some() {
                                info!("Received request with trace_id: '{trace_id}'");
                            } else {
                                info!("Received request without trace_id. Assigned: '{trace_id}'");
                            }

                            span.record("trace_id", trace_id.to_string());
                        } else {
                            error!("Unable to recover TraceId?");
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
            ))
}
