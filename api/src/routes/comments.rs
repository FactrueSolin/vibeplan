use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use validator::Validate;

use crate::{
    app::{AppState, comment},
    dto::comment::{
        CommentListResponse, CommentResponse, CreateCommentRequest, UpdateCommentRequest,
    },
    error::{ApiError, ApiResult},
    routes::RequestId,
};

#[utoipa::path(
    get,
    path = "/api/v1/tasks/{taskId}/comments",
    tag = "Comments",
    operation_id = "listTaskComments",
    params(("taskId" = String, Path, description = "Task id")),
    responses((status = 200, body = CommentListResponse))
)]
pub async fn list_comments(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    request_id: RequestId,
) -> ApiResult<Json<CommentListResponse>> {
    let comments = comment::list_comments(&state, &task_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(CommentListResponse {
        data: comments,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/tasks/{taskId}/comments",
    tag = "Comments",
    operation_id = "createTaskComment",
    params(("taskId" = String, Path, description = "Task id")),
    request_body = CreateCommentRequest,
    responses((status = 201, body = CommentResponse))
)]
pub async fn create_comment(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    request_id: RequestId,
    Json(payload): Json<CreateCommentRequest>,
) -> ApiResult<(StatusCode, Json<CommentResponse>)> {
    payload
        .validate()
        .map_err(ApiError::from)
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    let comment = comment::create_comment(&state, &task_id, payload)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok((
        StatusCode::CREATED,
        Json(CommentResponse {
            data: comment,
            meta: request_id.into_meta(),
        }),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/comments/{commentId}",
    tag = "Comments",
    operation_id = "updateComment",
    params(("commentId" = String, Path, description = "Comment id")),
    request_body = UpdateCommentRequest,
    responses((status = 200, body = CommentResponse))
)]
pub async fn update_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<String>,
    request_id: RequestId,
    Json(payload): Json<UpdateCommentRequest>,
) -> ApiResult<Json<CommentResponse>> {
    payload
        .validate()
        .map_err(ApiError::from)
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    let comment = comment::update_comment(&state, &comment_id, payload)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(CommentResponse {
        data: comment,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/comments/{commentId}",
    tag = "Comments",
    operation_id = "deleteComment",
    params(("commentId" = String, Path, description = "Comment id")),
    responses((status = 204))
)]
pub async fn delete_comment(
    State(state): State<AppState>,
    Path(comment_id): Path<String>,
    request_id: RequestId,
) -> ApiResult<StatusCode> {
    comment::delete_comment(&state, &comment_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0))?;
    Ok(StatusCode::NO_CONTENT)
}
