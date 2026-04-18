use sea_orm::{ActiveValue::Set, TransactionTrait};
use serde_json::json;

use crate::{
    app::{
        AppState, POSITION_STEP, ensure_non_blank, new_id, normalize_slug, now_rfc3339,
        project_to_dto, write_activity_log,
    },
    dto::project::{CreateProjectRequest, PaginationQuery, ProjectDto, UpdateProjectRequest},
    entity::{project, task_status},
    error::{ApiError, ApiResult},
    repository::{project::ProjectRepository, status::StatusRepository},
};

const DEFAULT_STATUSES: [(&str, &str, bool); 3] = [
    ("Todo", "#64748b", false),
    ("In Progress", "#0f766e", false),
    ("Done", "#16a34a", true),
];

pub async fn list_projects(
    state: &AppState,
    query: &PaginationQuery,
) -> ApiResult<(Vec<ProjectDto>, u64, u64, u64)> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let (projects, total) = ProjectRepository::list_paginated(&state.db, page, page_size).await?;
    Ok((
        projects.into_iter().map(project_to_dto).collect(),
        page,
        page_size,
        total,
    ))
}

pub async fn create_project(
    state: &AppState,
    request: CreateProjectRequest,
) -> ApiResult<ProjectDto> {
    ensure_non_blank("name", &request.name)?;
    ensure_non_blank("slug", &request.slug)?;
    let slug = normalize_slug(&request.slug)?;
    let now = now_rfc3339();
    let tx = state.db.begin().await?;

    let project = ProjectRepository::create(
        &tx,
        project::ActiveModel {
            id: Set(new_id()),
            name: Set(request.name.trim().to_owned()),
            slug: Set(slug),
            description: Set(request.description.map(|value| value.trim().to_owned())),
            color: Set(request.color.unwrap_or_else(|| "#2563eb".to_owned())),
            created_at: Set(now.clone()),
            updated_at: Set(now.clone()),
        },
    )
    .await?;

    for (index, (name, color, is_done)) in DEFAULT_STATUSES.into_iter().enumerate() {
        StatusRepository::create(
            &tx,
            task_status::ActiveModel {
                id: Set(new_id()),
                project_id: Set(project.id.clone()),
                name: Set(name.to_owned()),
                color: Set(color.to_owned()),
                sort_order: Set((index as i64 + 1) * POSITION_STEP),
                is_done: Set(is_done),
                is_hidden: Set(false),
                created_at: Set(now.clone()),
                updated_at: Set(now.clone()),
            },
        )
        .await?;
    }

    write_activity_log(
        &tx,
        &project.id,
        None,
        "project.created",
        json!({ "projectId": project.id }),
    )
    .await?;

    tx.commit().await?;
    Ok(project_to_dto(project))
}

pub async fn get_project(state: &AppState, project_id: &str) -> ApiResult<ProjectDto> {
    let project = ProjectRepository::find_by_id(&state.db, project_id)
        .await?
        .ok_or_else(|| ApiError::not_found("project not found"))?;
    Ok(project_to_dto(project))
}

pub async fn update_project(
    state: &AppState,
    project_id: &str,
    request: UpdateProjectRequest,
) -> ApiResult<ProjectDto> {
    let project = ProjectRepository::find_by_id(&state.db, project_id)
        .await?
        .ok_or_else(|| ApiError::not_found("project not found"))?;

    let mut active = ProjectRepository::active_model_from(&project);
    if let Some(name) = request.name {
        ensure_non_blank("name", &name)?;
        active.name = Set(name.trim().to_owned());
    }
    if let Some(slug) = request.slug {
        ensure_non_blank("slug", &slug)?;
        active.slug = Set(normalize_slug(&slug)?);
    }
    if let Some(description) = request.description {
        active.description = Set(description.map(|value| value.trim().to_owned()));
    }
    if let Some(color) = request.color {
        active.color = Set(color);
    }
    active.updated_at = Set(now_rfc3339());

    let tx = state.db.begin().await?;
    let updated = ProjectRepository::update(&tx, active).await?;
    write_activity_log(
        &tx,
        &updated.id,
        None,
        "project.updated",
        json!({ "projectId": updated.id }),
    )
    .await?;
    tx.commit().await?;

    Ok(project_to_dto(updated))
}

pub async fn delete_project(state: &AppState, project_id: &str) -> ApiResult<()> {
    let deleted = ProjectRepository::delete(&state.db, project_id).await?;
    if deleted == 0 {
        return Err(ApiError::not_found("project not found"));
    }
    Ok(())
}
