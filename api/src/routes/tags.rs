use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use validator::Validate;

use crate::{
    app::{AppState, tag},
    dto::{
        tag::{CreateTagRequest, TagListResponse, TagResponse, UpdateTagRequest},
        task::TaskResponse,
    },
    error::{ApiError, ApiResult},
    routes::RequestId,
};

#[utoipa::path(
    get,
    path = "/api/v1/projects/{projectId}/tags",
    tag = "Tags",
    operation_id = "listProjectTags",
    params(("projectId" = String, Path, description = "Project id")),
    responses((status = 200, body = TagListResponse))
)]
pub async fn list_tags(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    request_id: RequestId,
) -> ApiResult<Json<TagListResponse>> {
    let tags = tag::list_tags(&state, &project_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(TagListResponse {
        data: tags,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{projectId}/tags",
    tag = "Tags",
    operation_id = "createProjectTag",
    params(("projectId" = String, Path, description = "Project id")),
    request_body = CreateTagRequest,
    responses((status = 201, body = TagResponse))
)]
pub async fn create_tag(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    request_id: RequestId,
    Json(payload): Json<CreateTagRequest>,
) -> ApiResult<(StatusCode, Json<TagResponse>)> {
    payload
        .validate()
        .map_err(ApiError::from)
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    let tag = tag::create_tag(&state, &project_id, payload)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok((
        StatusCode::CREATED,
        Json(TagResponse {
            data: tag,
            meta: request_id.into_meta(),
        }),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/tags/{tagId}",
    tag = "Tags",
    operation_id = "updateTag",
    params(("tagId" = String, Path, description = "Tag id")),
    request_body = UpdateTagRequest,
    responses((status = 200, body = TagResponse))
)]
pub async fn update_tag(
    State(state): State<AppState>,
    Path(tag_id): Path<String>,
    request_id: RequestId,
    Json(payload): Json<UpdateTagRequest>,
) -> ApiResult<Json<TagResponse>> {
    payload
        .validate()
        .map_err(ApiError::from)
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    let tag = tag::update_tag(&state, &tag_id, payload)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(TagResponse {
        data: tag,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/tags/{tagId}",
    tag = "Tags",
    operation_id = "deleteTag",
    params(("tagId" = String, Path, description = "Tag id")),
    responses((status = 204))
)]
pub async fn delete_tag(
    State(state): State<AppState>,
    Path(tag_id): Path<String>,
    request_id: RequestId,
) -> ApiResult<StatusCode> {
    tag::delete_tag(&state, &tag_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0))?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/api/v1/tasks/{taskId}/tags/{tagId}",
    tag = "Tags",
    operation_id = "attachTaskTag",
    params(
        ("taskId" = String, Path, description = "Task id"),
        ("tagId" = String, Path, description = "Tag id")
    ),
    responses((status = 200, body = TaskResponse))
)]
pub async fn attach_tag(
    State(state): State<AppState>,
    Path((task_id, tag_id)): Path<(String, String)>,
    request_id: RequestId,
) -> ApiResult<Json<TaskResponse>> {
    let task = tag::attach_tag(&state, &task_id, &tag_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(TaskResponse {
        data: task,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/tasks/{taskId}/tags/{tagId}",
    tag = "Tags",
    operation_id = "detachTaskTag",
    params(
        ("taskId" = String, Path, description = "Task id"),
        ("tagId" = String, Path, description = "Tag id")
    ),
    responses((status = 200, body = TaskResponse))
)]
pub async fn detach_tag(
    State(state): State<AppState>,
    Path((task_id, tag_id)): Path<(String, String)>,
    request_id: RequestId,
) -> ApiResult<Json<TaskResponse>> {
    let task = tag::detach_tag(&state, &task_id, &tag_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(TaskResponse {
        data: task,
        meta: request_id.into_meta(),
    }))
}
