use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use validator::Validate;

use crate::{
    app::{AppState, project},
    dto::project::{
        CreateProjectRequest, PaginationQuery, ProjectListResponse, ProjectResponse,
        UpdateProjectRequest,
    },
    error::{ApiError, ApiResult},
    routes::RequestId,
};

#[utoipa::path(
    get,
    path = "/api/v1/projects",
    tag = "Projects",
    operation_id = "listProjects",
    params(PaginationQuery),
    responses(
        (status = 200, body = ProjectListResponse),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope)
    )
)]
pub async fn list_projects(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
    request_id: RequestId,
) -> ApiResult<Json<ProjectListResponse>> {
    let (items, page, page_size, total) = project::list_projects(&state, &query).await?;
    Ok(Json(ProjectListResponse {
        data: items,
        meta: request_id.into_meta().with_page(page, page_size, total),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/projects",
    tag = "Projects",
    operation_id = "createProject",
    request_body = CreateProjectRequest,
    responses(
        (status = 201, body = ProjectResponse),
        (status = 409, body = crate::error::ErrorEnvelope),
        (status = 422, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope)
    )
)]
pub async fn create_project(
    State(state): State<AppState>,
    request_id: RequestId,
    Json(payload): Json<CreateProjectRequest>,
) -> ApiResult<(StatusCode, Json<ProjectResponse>)> {
    payload
        .validate()
        .map_err(ApiError::from)
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    let project = project::create_project(&state, payload)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok((
        StatusCode::CREATED,
        Json(ProjectResponse {
            data: project,
            meta: request_id.into_meta(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{projectId}",
    tag = "Projects",
    operation_id = "getProject",
    params(("projectId" = String, Path, description = "Project id")),
    responses(
        (status = 200, body = ProjectResponse),
        (status = 404, body = crate::error::ErrorEnvelope)
    )
)]
pub async fn get_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    request_id: RequestId,
) -> ApiResult<Json<ProjectResponse>> {
    let project = project::get_project(&state, &project_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(ProjectResponse {
        data: project,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    patch,
    path = "/api/v1/projects/{projectId}",
    tag = "Projects",
    operation_id = "updateProject",
    params(("projectId" = String, Path, description = "Project id")),
    request_body = UpdateProjectRequest,
    responses(
        (status = 200, body = ProjectResponse),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 409, body = crate::error::ErrorEnvelope),
        (status = 422, body = crate::error::ErrorEnvelope)
    )
)]
pub async fn update_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    request_id: RequestId,
    Json(payload): Json<UpdateProjectRequest>,
) -> ApiResult<Json<ProjectResponse>> {
    payload
        .validate()
        .map_err(ApiError::from)
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    let project = project::update_project(&state, &project_id, payload)
        .await
        .map_err(|err| err.with_request_id(request_id.0.clone()))?;
    Ok(Json(ProjectResponse {
        data: project,
        meta: request_id.into_meta(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{projectId}",
    tag = "Projects",
    operation_id = "deleteProject",
    params(("projectId" = String, Path, description = "Project id")),
    responses(
        (status = 204, description = "Project deleted"),
        (status = 404, body = crate::error::ErrorEnvelope)
    )
)]
pub async fn delete_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    request_id: RequestId,
) -> ApiResult<StatusCode> {
    project::delete_project(&state, &project_id)
        .await
        .map_err(|err| err.with_request_id(request_id.0))?;
    Ok(StatusCode::NO_CONTENT)
}
