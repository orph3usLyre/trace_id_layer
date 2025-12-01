# trace_id_layer

A Tower middleware layer for transforming trace_ids into spans, or creating them when not provided

## Exposed functions:

**`create_trace_id_layer()`** - Creates a tracing layer using the default `x-trace-id` header.
**`create_trace_id_layer_with_header(header_name)`** - Creates a tracing layer with a custom trace ID header name.

Both functions automatically extract or generate trace IDs for HTTP requests and log request lifecycle events.

## Features

- Extracts trace IDs from incoming headers (`x-trace-id` by default)
- Supports custom header names for trace ID propagation
- Generates UUIDv7 trace IDs when none is provided
- Logs request latency, stream duration, and errors
- Attaches trace IDs to tracing spans for correlation

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
trace_id_layer = "0.1.0"
```

### Basic Usage 

```rust
use axum::Router;
use trace_id_layer::create_trace_id_layer;

let app = Router::new()
    .route("/", get(handler))
    .layer(create_trace_id_layer());
```
