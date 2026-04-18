use sea_orm::{ActiveValue::Set, TransactionTrait};
use serde_json::json;

use crate::{
    app::{AppState, ensure_non_blank, new_id, now_rfc3339, tag_to_dto, write_activity_log},
    dto::tag::{CreateTagRequest, TagDto, UpdateTagRequest},
    entity::tag,
    error::{ApiError, ApiResult},
    repository::{project::ProjectRepository, tag::TagRepository, task::TaskRepository},
};

pub async fn list_tags(state: &AppState, project_id: &str) -> ApiResult<Vec<TagDto>> {
    ProjectRepository::find_by_id(&state.db, project_id)
        .await?
        .ok_or_else(|| ApiError::not_found("project not found"))?;
    let tags = TagRepository::list_by_project(&state.db, project_id).await?;
    Ok(tags.into_iter().map(tag_to_dto).collect())
}

pub async fn create_tag(
    state: &AppState,
    project_id: &str,
    request: CreateTagRequest,
) -> ApiResult<TagDto> {
    ProjectRepository::find_by_id(&state.db, project_id)
        .await?
        .ok_or_else(|| ApiError::not_found("project not found"))?;
    ensure_non_blank("name", &request.name)?;
    let now = now_rfc3339();
    let tx = state.db.begin().await?;
    let tag = TagRepository::create(
        &tx,
        tag::ActiveModel {
            id: Set(new_id()),
            project_id: Set(project_id.to_owned()),
            name: Set(request.name.trim().to_owned()),
            color: Set(request.color),
            created_at: Set(now.clone()),
            updated_at: Set(now.clone()),
        },
    )
    .await?;
    write_activity_log(
        &tx,
        project_id,
        None,
        "tag.created",
        json!({ "tagId": tag.id }),
    )
    .await?;
    tx.commit().await?;
    Ok(tag_to_dto(tag))
}

pub async fn update_tag(
    state: &AppState,
    tag_id: &str,
    request: UpdateTagRequest,
) -> ApiResult<TagDto> {
    let tag = TagRepository::find_by_id(&state.db, tag_id)
        .await?
        .ok_or_else(|| ApiError::not_found("tag not found"))?;
    let mut active = TagRepository::active_model_from(&tag);
    if let Some(name) = request.name {
        ensure_non_blank("name", &name)?;
        active.name = Set(name.trim().to_owned());
    }
    if let Some(color) = request.color {
        active.color = Set(color);
    }
    active.updated_at = Set(now_rfc3339());

    let tx = state.db.begin().await?;
    let updated = TagRepository::update(&tx, active).await?;
    write_activity_log(
        &tx,
        &updated.project_id,
        None,
        "tag.updated",
        json!({ "tagId": updated.id }),
    )
    .await?;
    tx.commit().await?;
    Ok(tag_to_dto(updated))
}

pub async fn delete_tag(state: &AppState, tag_id: &str) -> ApiResult<()> {
    let tag = TagRepository::find_by_id(&state.db, tag_id)
        .await?
        .ok_or_else(|| ApiError::not_found("tag not found"))?;
    let tx = state.db.begin().await?;
    let deleted = TagRepository::delete(&tx, tag_id).await?;
    if deleted == 0 {
        return Err(ApiError::not_found("tag not found"));
    }
    write_activity_log(
        &tx,
        &tag.project_id,
        None,
        "tag.deleted",
        json!({ "tagId": tag_id }),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn attach_tag(
    state: &AppState,
    task_id: &str,
    tag_id: &str,
) -> ApiResult<crate::dto::task::TaskDto> {
    let task = TaskRepository::find_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| ApiError::not_found("task not found"))?;
    let tag = TagRepository::find_by_id(&state.db, tag_id)
        .await?
        .ok_or_else(|| ApiError::not_found("tag not found"))?;
    if task.project_id != tag.project_id {
        return Err(ApiError::conflict(
            "task and tag must belong to the same project",
        ));
    }

    let tx = state.db.begin().await?;
    TagRepository::attach_tag(&tx, &task.project_id, task_id, tag_id, &now_rfc3339()).await?;
    write_activity_log(
        &tx,
        &task.project_id,
        Some(task_id),
        "task.tag_attached",
        json!({ "tagId": tag_id }),
    )
    .await?;
    tx.commit().await?;
    crate::app::task::get_task(state, task_id).await
}

pub async fn detach_tag(
    state: &AppState,
    task_id: &str,
    tag_id: &str,
) -> ApiResult<crate::dto::task::TaskDto> {
    let task = TaskRepository::find_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| ApiError::not_found("task not found"))?;
    let tx = state.db.begin().await?;
    TagRepository::detach_tag(&tx, task_id, tag_id).await?;
    write_activity_log(
        &tx,
        &task.project_id,
        Some(task_id),
        "task.tag_detached",
        json!({ "tagId": tag_id }),
    )
    .await?;
    tx.commit().await?;
    crate::app::task::get_task(state, task_id).await
}
