mod billing;
mod webhook;

use std::time::Duration;

use crate::state::AxumState;
use axum::{
    Router,
    extract::MatchedPath,
    http::{Request, Response},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info_span;

pub fn router() -> Router<AxumState> {
    Router::new()
        .nest("/billing", billing::router())
        .nest("/webhook", webhook::router())
        .layer(CorsLayer::very_permissive())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "response",
                        method = ?request.method(),
                        matched_path,
                        some_other_field = tracing::field::Empty,
                    )
                })
                .on_response(
                    |response: &Response<_>, latency: Duration, span: &tracing::Span| {
                        span.record("status", response.status().as_u16());
                        span.record("latency", latency.as_millis());
                        span.record(
                            "content_length",
                            response
                                .headers()
                                .get("content-length")
                                .and_then(|v| v.to_str().ok())
                                .and_then(|s| s.parse::<u64>().ok())
                                .unwrap_or(0),
                        );
                        span.record(
                            "matched_path",
                            response
                                .extensions()
                                .get::<MatchedPath>()
                                .map(MatchedPath::as_str),
                        );

                        tracing::info!(
                            "response: {} in {}ms",
                            response.status(),
                            latency.as_millis()
                        );
                    },
                ),
        )
}
