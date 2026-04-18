use std::collections::HashSet;

use sea_orm::{ActiveValue::Set, TransactionTrait};
use serde_json::json;

use crate::{
    app::{
        AppState, POSITION_STEP, ensure_non_blank, new_id, now_rfc3339, status_to_dto,
        write_activity_log,
    },
    dto::status::{CreateStatusRequest, ReorderStatusesRequest, StatusDto, UpdateStatusRequest},
    entity::task_status,
    error::{ApiError, ApiResult},
    repository::{project::ProjectRepository, status::StatusRepository},
};

pub async fn list_statuses(state: &AppState, project_id: &str) -> ApiResult<Vec<StatusDto>> {
    ensure_project_exists(state, project_id).await?;
    let statuses = StatusRepository::list_by_project(&state.db, project_id).await?;
    Ok(statuses.into_iter().map(status_to_dto).collect())
}

pub async fn create_status(
    state: &AppState,
    project_id: &str,
    request: CreateStatusRequest,
) -> ApiResult<StatusDto> {
    ensure_project_exists(state, project_id).await?;
    ensure_non_blank("name", &request.name)?;
    let sort_order = StatusRepository::max_sort_order(&state.db, project_id)
        .await?
        .unwrap_or(0)
        + POSITION_STEP;
    let now = now_rfc3339();
    let tx = state.db.begin().await?;
    let status = StatusRepository::create(
        &tx,
        task_status::ActiveModel {
            id: Set(new_id()),
            project_id: Set(project_id.to_owned()),
            name: Set(request.name.trim().to_owned()),
            color: Set(request.color),
            sort_order: Set(sort_order),
            is_done: Set(request.is_done.unwrap_or(false)),
            is_hidden: Set(request.is_hidden.unwrap_or(false)),
            created_at: Set(now.clone()),
            updated_at: Set(now.clone()),
        },
    )
    .await?;
    write_activity_log(
        &tx,
        project_id,
        None,
        "status.created",
        json!({ "statusId": status.id }),
    )
    .await?;
    tx.commit().await?;
    Ok(status_to_dto(status))
}

pub async fn update_status(
    state: &AppState,
    status_id: &str,
    request: UpdateStatusRequest,
) -> ApiResult<StatusDto> {
    let status = StatusRepository::find_by_id(&state.db, status_id)
        .await?
        .ok_or_else(|| ApiError::not_found("status not found"))?;
    let mut active = StatusRepository::active_model_from(&status);
    if let Some(name) = request.name {
        ensure_non_blank("name", &name)?;
        active.name = Set(name.trim().to_owned());
    }
    if let Some(color) = request.color {
        active.color = Set(color);
    }
    if let Some(is_done) = request.is_done {
        active.is_done = Set(is_done);
    }
    if let Some(is_hidden) = request.is_hidden {
        active.is_hidden = Set(is_hidden);
    }
    active.updated_at = Set(now_rfc3339());

    let tx = state.db.begin().await?;
    let updated = StatusRepository::update(&tx, active).await?;
    write_activity_log(
        &tx,
        &updated.project_id,
        None,
        "status.updated",
        json!({ "statusId": updated.id }),
    )
    .await?;
    tx.commit().await?;
    Ok(status_to_dto(updated))
}

pub async fn delete_status(state: &AppState, status_id: &str) -> ApiResult<()> {
    let status = StatusRepository::find_by_id(&state.db, status_id)
        .await?
        .ok_or_else(|| ApiError::not_found("status not found"))?;

    let active_task_count = StatusRepository::count_active_tasks(&state.db, status_id).await?;
    if active_task_count > 0 {
        return Err(ApiError::conflict(
            "status contains active tasks and cannot be deleted",
        ));
    }

    let tx = state.db.begin().await?;
    StatusRepository::delete_archived_tasks(&tx, status_id).await?;
    let deleted = StatusRepository::delete(&tx, status_id).await?;
    if deleted == 0 {
        return Err(ApiError::not_found("status not found"));
    }
    write_activity_log(
        &tx,
        &status.project_id,
        None,
        "status.deleted",
        json!({ "statusId": status.id }),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn reorder_statuses(
    state: &AppState,
    project_id: &str,
    request: ReorderStatusesRequest,
) -> ApiResult<Vec<StatusDto>> {
    let statuses = StatusRepository::list_by_project(&state.db, project_id).await?;
    let expected: HashSet<&str> = statuses.iter().map(|status| status.id.as_str()).collect();
    let received: HashSet<&str> = request
        .ordered_status_ids
        .iter()
        .map(String::as_str)
        .collect();
    if expected != received || request.ordered_status_ids.len() != statuses.len() {
        return Err(ApiError::validation(
            "orderedStatusIds must contain every project status exactly once",
        ));
    }

    let tx = state.db.begin().await?;
    for (index, status_id) in request.ordered_status_ids.iter().enumerate() {
        let status = statuses
            .iter()
            .find(|status| status.id == *status_id)
            .expect("status set already validated");
        let mut active = StatusRepository::active_model_from(status);
        active.sort_order = Set((index as i64 + 1) * POSITION_STEP);
        active.updated_at = Set(now_rfc3339());
        StatusRepository::update(&tx, active).await?;
    }

    write_activity_log(
        &tx,
        project_id,
        None,
        "status.reordered",
        json!({ "orderedStatusIds": request.ordered_status_ids }),
    )
    .await?;
    tx.commit().await?;

    list_statuses(state, project_id).await
}

async fn ensure_project_exists(state: &AppState, project_id: &str) -> ApiResult<()> {
    ProjectRepository::find_by_id(&state.db, project_id)
        .await?
        .ok_or_else(|| ApiError::not_found("project not found"))?;
    Ok(())
}
