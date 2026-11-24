use axum::{
    extract::{DefaultBodyLimit, Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use ofs_core::calculation::{estimate_fixed_frame, CostEstimate};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use subtle::ConstantTimeEq;

pub fn router(token: &str) -> Result<Router, &'static str> {
    if token.len() < 32 || !token.bytes().all(|b| b.is_ascii_graphic()) {
        return Err("OFS_API_TOKEN requires at least 32 printable, non-space ASCII characters");
    }
    let estimate_api = Router::new()
        .route("/api/v1/estimates/fixed-frame", post(estimate))
        .route_layer(middleware::from_fn_with_state(
            Arc::<str>::from(token),
            authorize,
        ))
        .layer(DefaultBodyLimit::max(4096));
    Ok(estimate_api.route(
        "/api/v1/health",
        get(|| async { Json(json!({"status":"ok", "version":env!("CARGO_PKG_VERSION")})) }),
    ))
}

async fn authorize(State(expected): State<Arc<str>>, req: Request, next: Next) -> Response {
    let provided = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split_once(' '))
        .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("Bearer"))
        .map(|(_, value)| value.as_bytes())
        .unwrap_or_default();
    if !bool::from(expected.as_bytes().ct_eq(provided)) {
        return (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
        )
            .into_response();
    }
    next.run(req).await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FixedFrame {
    width_mm: f64,
    height_mm: f64,
}

async fn estimate(
    Json(input): Json<FixedFrame>,
) -> Result<Json<CostEstimate>, (StatusCode, &'static str)> {
    estimate_fixed_frame(input.width_mm, input.height_mm)
        .map(Json)
        .ok_or((
            StatusCode::UNPROCESSABLE_ENTITY,
            "widthMm must be 200–6000 and heightMm 200–4000",
        ))
}
