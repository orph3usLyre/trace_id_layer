# trace_id_layer

Tower middleware that extracts or generates trace IDs for HTTP requests and attaches them to tracing spans.

## How it works

`add_trace_id_middleware()` wraps an Axum router with `tower-http`'s `TraceLayer`:

1. **Checks for `x-trace-id` header** - If present, uses that value as the trace ID
2. **Generates UUIDv7 if missing** - Creates a new trace ID when none is provided
3. **Records to span** - Attaches trace ID to the `http-request` span for log correlation
4. **Logs lifecycle events** - Response latency, stream duration, and errors

## Usage

```rust
use axum::Router;
use trace_id_layer::add_trace_id_middleware;

let router = Router::new()
    .route("/", get(handler));
    
let router = add_trace_id_middleware(router);
```
