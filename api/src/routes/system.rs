use axum::Json;
use serde_json::{Value, json};

use crate::{dto::common::ApiMeta, openapi, routes::RequestId};

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "System",
    operation_id = "healthCheck",
    responses(
        (status = 200, description = "Health check", body = serde_json::Value)
    )
)]
pub async fn health(request_id: RequestId) -> Json<Value> {
    Json(json!({
        "data": { "status": "ok" },
        "meta": ApiMeta::new(request_id.0),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/openapi.json",
    tag = "System",
    operation_id = "getOpenApi",
    responses(
        (status = 200, description = "OpenAPI document", body = serde_json::Value)
    )
)]
pub async fn openapi() -> Json<Value> {
    Json(openapi::openapi_json_value())
}
