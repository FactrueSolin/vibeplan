use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use validator::Validate;

use crate::{
    app::{AppState, status},
    dto::status::{
        CreateStatusRequest, ReorderStatusesRequest, StatusListResponse, StatusResponse,
        UpdateStatusRequest,
    },
    error::{ApiError, ApiResult},
    routes::RequestId,
};

#[utoipa::path(
    get,
    path = "/api/v1/projects/{projectId}/statuses",
    tag = "Statuses",
    operation_id = "listProjectStatuses",
    params(("projectId" = String, Path, description = "Project id")),
    responses((status = 200, body = StatusListResponse))
)]
pub async fn list_statuses(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    request_id: RequestId,
) -> ApiResult<Json<StatusListResponse>> {
    let statuses = status::list_statuses(&state, &project_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(StatusListResponse {
        data: statuses,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{projectId}/statuses",
    tag = "Statuses",
    operation_id = "createProjectStatus",
    params(("projectId" = String, Path, description = "Project id")),
    request_body = CreateStatusRequest,
    responses((status = 201, body = StatusResponse))
)]
pub async fn create_status(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    request_id: RequestId,
    Json(payload): Json<CreateStatusRequest>,
) -> ApiResult<(StatusCode, Json<StatusResponse>)> {
    payload
        .validate()
        .map_err(ApiError::from)
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    let status = status::create_status(&state, &project_id, payload)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok((
        StatusCode::CREATED,
        Json(StatusResponse {
            data: status,
            meta: request_id.into_meta(),
        }),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/statuses/{statusId}",
    tag = "Statuses",
    operation_id = "updateStatus",
    params(("statusId" = String, Path, description = "Status id")),
    request_body = UpdateStatusRequest,
    responses((status = 200, body = StatusResponse))
)]
pub async fn update_status(
    State(state): State<AppState>,
    Path(status_id): Path<String>,
    request_id: RequestId,
    Json(payload): Json<UpdateStatusRequest>,
) -> ApiResult<Json<StatusResponse>> {
    payload
        .validate()
        .map_err(ApiError::from)
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    let status = status::update_status(&state, &status_id, payload)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(StatusResponse {
        data: status,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/statuses/{statusId}",
    tag = "Statuses",
    operation_id = "deleteStatus",
    params(("statusId" = String, Path, description = "Status id")),
    responses((status = 204))
)]
pub async fn delete_status(
    State(state): State<AppState>,
    Path(status_id): Path<String>,
    request_id: RequestId,
) -> ApiResult<StatusCode> {
    status::delete_status(&state, &status_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0))?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{projectId}/statuses/reorder",
    tag = "Statuses",
    operation_id = "reorderProjectStatuses",
    params(("projectId" = String, Path, description = "Project id")),
    request_body = ReorderStatusesRequest,
    responses((status = 200, body = StatusListResponse))
)]
pub async fn reorder_statuses(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    request_id: RequestId,
    Json(payload): Json<ReorderStatusesRequest>,
) -> ApiResult<Json<StatusListResponse>> {
    let statuses = status::reorder_statuses(&state, &project_id, payload)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(StatusListResponse {
        data: statuses,
        meta: request_id.into_meta(),
    }))
}
