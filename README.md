# trace_id_layer

Tower middleware that extracts or generates trace IDs for HTTP requests and attaches them to tracing spans.

## How it works

`add_trace_id_middleware()` wraps an Axum router with `tower-http`'s `TraceLayer`:

1. **Checks for `x-trace-id` header** - If present, uses that value as the trace ID
2. **Generates UUIDv7 if missing** - Creates a new trace ID when none is provided
3. **Records to span** - Attaches trace ID to the `http-request` span for log correlation
4. **Exposes to handlers** - Makes trace ID available via the `TraceId` extractor
5. **Logs lifecycle events** - Response latency, stream duration, and errors

## Usage

### Basic Setup

```rust
use axum::Router;
use trace_id_layer::add_trace_id_middleware;

let router = Router::new()
    .route("/", get(handler));
    
let router = add_trace_id_middleware(router);
```

### Accessing Trace ID in Handlers

Use the `TraceId` extractor to access the trace ID in your handlers:

```rust
use axum::http::StatusCode;
use trace_id_layer::TraceId;

async fn my_handler(trace_id: TraceId) -> String {
    // The trace_id is available as trace_id.0
    format!("Request trace ID: {}", trace_id.0)
}

async fn another_handler(trace_id: TraceId) -> StatusCode {
    tracing::info!("Processing request with trace_id: {}", trace_id.0);
    StatusCode::OK
}
```
