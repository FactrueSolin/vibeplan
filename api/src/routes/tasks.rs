use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use validator::Validate;

use crate::{
    app::{AppState, task},
    dto::task::{
        BoardQuery, BoardResponse, CreateTaskRequest, ReorderTasksRequest, TaskListQuery,
        TaskListResponse, TaskResponse, UpdateTaskRequest,
    },
    error::{ApiError, ApiResult},
    routes::RequestId,
};

#[utoipa::path(
    get,
    path = "/api/v1/projects/{projectId}/board",
    tag = "Tasks",
    operation_id = "getBoardSnapshot",
    params(
        ("projectId" = String, Path, description = "Project id"),
        BoardQuery
    ),
    responses((status = 200, body = BoardResponse))
)]
pub async fn get_board_snapshot(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Query(query): Query<BoardQuery>,
    request_id: RequestId,
) -> ApiResult<Json<BoardResponse>> {
    let board =
        task::get_board_snapshot(&state, &project_id, query.include_archived.unwrap_or(false))
            .await
            .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(BoardResponse {
        data: board,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{projectId}/tasks",
    tag = "Tasks",
    operation_id = "listProjectTasks",
    params(
        ("projectId" = String, Path, description = "Project id"),
        TaskListQuery
    ),
    responses((status = 200, body = TaskListResponse))
)]
pub async fn list_tasks(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Query(query): Query<TaskListQuery>,
    request_id: RequestId,
) -> ApiResult<Json<TaskListResponse>> {
    let (tasks, page, page_size, total) = task::list_tasks(&state, &project_id, &query)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(TaskListResponse {
        data: tasks,
        meta: request_id.into_meta().with_page(page, page_size, total),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{projectId}/tasks",
    tag = "Tasks",
    operation_id = "createProjectTask",
    params(("projectId" = String, Path, description = "Project id")),
    request_body = CreateTaskRequest,
    responses((status = 201, body = TaskResponse))
)]
pub async fn create_task(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    request_id: RequestId,
    Json(payload): Json<CreateTaskRequest>,
) -> ApiResult<(StatusCode, Json<TaskResponse>)> {
    payload
        .validate()
        .map_err(ApiError::from)
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    let task = task::create_task(&state, &project_id, payload)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok((
        StatusCode::CREATED,
        Json(TaskResponse {
            data: task,
            meta: request_id.into_meta(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/tasks/{taskId}",
    tag = "Tasks",
    operation_id = "getTask",
    params(("taskId" = String, Path, description = "Task id")),
    responses((status = 200, body = TaskResponse))
)]
pub async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    request_id: RequestId,
) -> ApiResult<Json<TaskResponse>> {
    let task = task::get_task(&state, &task_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(TaskResponse {
        data: task,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/tasks/{taskId}",
    tag = "Tasks",
    operation_id = "updateTask",
    params(("taskId" = String, Path, description = "Task id")),
    request_body = UpdateTaskRequest,
    responses((status = 200, body = TaskResponse))
)]
pub async fn update_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    request_id: RequestId,
    Json(payload): Json<UpdateTaskRequest>,
) -> ApiResult<Json<TaskResponse>> {
    payload
        .validate()
        .map_err(ApiError::from)
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    let task = task::update_task(&state, &task_id, payload)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(TaskResponse {
        data: task,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/tasks/{taskId}",
    tag = "Tasks",
    operation_id = "deleteTask",
    params(("taskId" = String, Path, description = "Task id")),
    responses((status = 204))
)]
pub async fn delete_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    request_id: RequestId,
) -> ApiResult<StatusCode> {
    task::delete_task(&state, &task_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0))?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/tasks/{taskId}/archive",
    tag = "Tasks",
    operation_id = "archiveTask",
    params(("taskId" = String, Path, description = "Task id")),
    responses((status = 200, body = TaskResponse))
)]
pub async fn archive_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    request_id: RequestId,
) -> ApiResult<Json<TaskResponse>> {
    let task = task::archive_task(&state, &task_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(TaskResponse {
        data: task,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/tasks/{taskId}/restore",
    tag = "Tasks",
    operation_id = "restoreTask",
    params(("taskId" = String, Path, description = "Task id")),
    responses((status = 200, body = TaskResponse))
)]
pub async fn restore_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
    request_id: RequestId,
) -> ApiResult<Json<TaskResponse>> {
    let task = task::restore_task(&state, &task_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(TaskResponse {
        data: task,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{projectId}/tasks/reorder",
    tag = "Tasks",
    operation_id = "reorderProjectTasks",
    params(("projectId" = String, Path, description = "Project id")),
    request_body = ReorderTasksRequest,
    responses((status = 200, body = TaskListResponse))
)]
pub async fn reorder_tasks(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    request_id: RequestId,
    Json(payload): Json<ReorderTasksRequest>,
) -> ApiResult<Json<TaskListResponse>> {
    let tasks = task::reorder_tasks(&state, &project_id, payload)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(TaskListResponse {
        data: tasks,
        meta: request_id.into_meta(),
    }))
}
