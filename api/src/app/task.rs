use std::collections::{HashMap, HashSet};

use sea_orm::{ActiveValue::Set, TransactionTrait};
use serde_json::json;

use crate::{
    app::{
        AppState, POSITION_STEP, new_id, now_rfc3339, project_to_dto, status_to_dto, tag_to_dto,
        task_tag_to_dto, task_to_dto, validate_date_range, write_activity_log,
    },
    dto::task::{
        BoardSnapshotDto, BoardSummaryDto, CreateTaskRequest, ReorderTasksRequest, TaskDto,
        TaskListQuery, UpdateTaskRequest,
    },
    entity::task,
    error::{ApiError, ApiResult},
    repository::{
        project::ProjectRepository, status::StatusRepository, tag::TagRepository,
        task::TaskRepository,
    },
};

pub async fn get_board_snapshot(
    state: &AppState,
    project_id: &str,
    include_archived: bool,
) -> ApiResult<BoardSnapshotDto> {
    let project = ProjectRepository::find_by_id(&state.db, project_id)
        .await?
        .ok_or_else(|| ApiError::not_found("project not found"))?;
    let statuses = StatusRepository::list_by_project(&state.db, project_id).await?;
    let tags = TagRepository::list_by_project(&state.db, project_id).await?;
    let tasks = TaskRepository::list_by_project(&state.db, project_id, include_archived).await?;
    let task_ids: Vec<String> = tasks.iter().map(|task| task.id.clone()).collect();
    let task_tags =
        TagRepository::list_task_tags_by_project(&state.db, project_id, Some(&task_ids)).await?;
    let tag_map = TagRepository::build_tag_map(&task_tags);

    let summary = BoardSummaryDto {
        active_task_count: TaskRepository::count_active(&state.db, project_id).await?,
        done_task_count: TaskRepository::count_done(&state.db, project_id).await?,
        archived_task_count: TaskRepository::count_archived(&state.db, project_id).await?,
    };

    Ok(BoardSnapshotDto {
        project: project_to_dto(project),
        statuses: statuses.into_iter().map(status_to_dto).collect(),
        tasks: tasks
            .into_iter()
            .map(|task| {
                let tags = tag_map.get(&task.id).cloned().unwrap_or_default();
                task_to_dto(task, tags)
            })
            .collect(),
        tags: tags.into_iter().map(tag_to_dto).collect(),
        task_tags: task_tags.into_iter().map(task_tag_to_dto).collect(),
        summary,
    })
}

pub async fn list_tasks(
    state: &AppState,
    project_id: &str,
    query: &TaskListQuery,
) -> ApiResult<(Vec<TaskDto>, u64, u64, u64)> {
    ProjectRepository::find_by_id(&state.db, project_id)
        .await?
        .ok_or_else(|| ApiError::not_found("project not found"))?;

    let (tasks, total) =
        TaskRepository::list_by_project_paginated(&state.db, project_id, query).await?;

    let task_ids: Vec<String> = tasks.iter().map(|task| task.id.clone()).collect();
    let tag_map = load_tag_map(state, project_id, &task_ids).await?;
    let items = tasks
        .into_iter()
        .map(|task| {
            task_to_dto(
                task.clone(),
                tag_map.get(&task.id).cloned().unwrap_or_default(),
            )
        })
        .collect();
    Ok((
        items,
        query.page.unwrap_or(1).max(1),
        query.page_size.unwrap_or(20).clamp(1, 100),
        total,
    ))
}

pub async fn create_task(
    state: &AppState,
    project_id: &str,
    request: CreateTaskRequest,
) -> ApiResult<TaskDto> {
    validate_date_range(&request.start_date, &request.due_date)?;
    let project = ProjectRepository::find_by_id(&state.db, project_id)
        .await?
        .ok_or_else(|| ApiError::not_found("project not found"))?;
    let status = StatusRepository::find_by_id(&state.db, &request.status_id)
        .await?
        .ok_or_else(|| ApiError::not_found("status not found"))?;
    if status.project_id != project.id {
        return Err(ApiError::conflict("status does not belong to the project"));
    }

    validate_tag_ids(state, project_id, &request.tag_ids).await?;

    let position = TaskRepository::max_position(&state.db, &request.status_id)
        .await?
        .unwrap_or(0)
        + POSITION_STEP;
    let now = now_rfc3339();
    let tx = state.db.begin().await?;
    let task = TaskRepository::create(
        &tx,
        task::ActiveModel {
            id: Set(new_id()),
            project_id: Set(project_id.to_owned()),
            status_id: Set(request.status_id),
            title: Set(request.title.trim().to_owned()),
            description: Set(request.description.map(|value| value.trim().to_owned())),
            priority: Set(request.priority),
            position: Set(position),
            start_date: Set(request.start_date),
            due_date: Set(request.due_date),
            completed_at: Set(if status.is_done {
                Some(now.clone())
            } else {
                None
            }),
            archived_at: Set(None),
            created_at: Set(now.clone()),
            updated_at: Set(now.clone()),
        },
    )
    .await?;
    TagRepository::replace_task_tags(&tx, project_id, &task.id, &request.tag_ids, &now).await?;
    write_activity_log(
        &tx,
        project_id,
        Some(&task.id),
        "task.created",
        json!({ "taskId": task.id }),
    )
    .await?;
    tx.commit().await?;

    get_task(state, &task.id).await
}

pub async fn get_task(state: &AppState, task_id: &str) -> ApiResult<TaskDto> {
    let task = TaskRepository::find_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| ApiError::not_found("task not found"))?;
    let tag_map = load_tag_map(state, &task.project_id, &[task.id.clone()]).await?;
    Ok(task_to_dto(
        task.clone(),
        tag_map.get(&task.id).cloned().unwrap_or_default(),
    ))
}

pub async fn update_task(
    state: &AppState,
    task_id: &str,
    request: UpdateTaskRequest,
) -> ApiResult<TaskDto> {
    let task = TaskRepository::find_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| ApiError::not_found("task not found"))?;
    let current_status = StatusRepository::find_by_id(&state.db, &task.status_id)
        .await?
        .ok_or_else(|| ApiError::not_found("status not found"))?;
    let target_status = if let Some(status_id) = request.status_id.as_ref() {
        let status = StatusRepository::find_by_id(&state.db, status_id)
            .await?
            .ok_or_else(|| ApiError::not_found("status not found"))?;
        if status.project_id != task.project_id {
            return Err(ApiError::conflict(
                "status does not belong to the task project",
            ));
        }
        status
    } else {
        current_status
    };

    let mut start_date = task.start_date.clone();
    let mut due_date = task.due_date.clone();
    if let Some(value) = request.start_date.clone() {
        start_date = value;
    }
    if let Some(value) = request.due_date.clone() {
        due_date = value;
    }
    validate_date_range(&start_date, &due_date)?;

    if let Some(tag_ids) = request.tag_ids.as_ref() {
        validate_tag_ids(state, &task.project_id, tag_ids).await?;
    }

    let mut active = TaskRepository::active_model_from(&task);
    if let Some(title) = request.title {
        active.title = Set(title.trim().to_owned());
    }
    if let Some(description) = request.description {
        active.description = Set(description.map(|value| value.trim().to_owned()));
    }
    if let Some(priority) = request.priority {
        active.priority = Set(priority);
    }
    if let Some(status_id) = request.status_id {
        let status_changed = task.status_id != status_id;
        active.status_id = Set(status_id);
        if status_changed {
            let position = TaskRepository::max_position(&state.db, &target_status.id)
                .await?
                .unwrap_or(0)
                + POSITION_STEP;
            active.position = Set(position);
        }
    }
    if let Some(value) = request.start_date {
        active.start_date = Set(value);
    }
    if let Some(value) = request.due_date {
        active.due_date = Set(value);
    }
    active.completed_at = Set(if target_status.is_done {
        task.completed_at.or_else(|| Some(now_rfc3339()))
    } else {
        None
    });
    active.updated_at = Set(now_rfc3339());

    let tx = state.db.begin().await?;
    let updated = TaskRepository::update(&tx, active).await?;
    if let Some(tag_ids) = request.tag_ids {
        TagRepository::replace_task_tags(&tx, &task.project_id, task_id, &tag_ids, &now_rfc3339())
            .await?;
    }
    write_activity_log(
        &tx,
        &task.project_id,
        Some(task_id),
        "task.updated",
        json!({ "taskId": task_id }),
    )
    .await?;
    tx.commit().await?;

    get_task(state, &updated.id).await
}

pub async fn delete_task(state: &AppState, task_id: &str) -> ApiResult<()> {
    let task = TaskRepository::find_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| ApiError::not_found("task not found"))?;
    if task.archived_at.is_none() {
        return Err(ApiError::invalid_operation(
            "task must be archived before it can be deleted",
        ));
    }
    let tx = state.db.begin().await?;
    let deleted = TaskRepository::delete(&tx, task_id).await?;
    if deleted == 0 {
        return Err(ApiError::not_found("task not found"));
    }
    write_activity_log(
        &tx,
        &task.project_id,
        Some(task_id),
        "task.deleted",
        json!({ "taskId": task_id }),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn archive_task(state: &AppState, task_id: &str) -> ApiResult<TaskDto> {
    set_archived(state, task_id, true).await
}

pub async fn restore_task(state: &AppState, task_id: &str) -> ApiResult<TaskDto> {
    set_archived(state, task_id, false).await
}

pub async fn reorder_tasks(
    state: &AppState,
    project_id: &str,
    request: ReorderTasksRequest,
) -> ApiResult<Vec<TaskDto>> {
    let moved_task = TaskRepository::find_by_id(&state.db, &request.moved_task_id)
        .await?
        .ok_or_else(|| ApiError::not_found("task not found"))?;
    if moved_task.project_id != project_id {
        return Err(ApiError::conflict("task does not belong to the project"));
    }
    let source_status = StatusRepository::find_by_id(&state.db, &request.source_status_id)
        .await?
        .ok_or_else(|| ApiError::not_found("source status not found"))?;
    let destination_status =
        StatusRepository::find_by_id(&state.db, &request.destination_status_id)
            .await?
            .ok_or_else(|| ApiError::not_found("destination status not found"))?;
    if source_status.project_id != project_id || destination_status.project_id != project_id {
        return Err(ApiError::conflict("status does not belong to the project"));
    }

    let source_tasks =
        TaskRepository::list_by_status(&state.db, &request.source_status_id, false).await?;
    let destination_tasks = if request.source_status_id == request.destination_status_id {
        source_tasks.clone()
    } else {
        TaskRepository::list_by_status(&state.db, &request.destination_status_id, false).await?
    };

    validate_reorder_payload(&moved_task, &request, &source_tasks, &destination_tasks)?;

    let tx = state.db.begin().await?;
    apply_destination_order(
        &tx,
        &request.destination_status_id,
        &request.ordered_task_ids,
        &moved_task,
        destination_status.is_done,
    )
    .await?;

    if request.source_status_id != request.destination_status_id {
        let remaining_ids: Vec<String> = source_tasks
            .into_iter()
            .filter(|task| task.id != moved_task.id)
            .map(|task| task.id)
            .collect();
        apply_source_order(&tx, &remaining_ids).await?;
    }

    write_activity_log(
        &tx,
        project_id,
        Some(&moved_task.id),
        "task.reordered",
        json!({
            "taskId": moved_task.id,
            "sourceStatusId": request.source_status_id,
            "destinationStatusId": request.destination_status_id,
            "orderedTaskIds": request.ordered_task_ids,
        }),
    )
    .await?;
    tx.commit().await?;

    let updated_tasks = if request.source_status_id == request.destination_status_id {
        TaskRepository::list_by_status(&state.db, &request.destination_status_id, false).await?
    } else {
        let mut tasks =
            TaskRepository::list_by_status(&state.db, &request.source_status_id, false).await?;
        tasks.extend(
            TaskRepository::list_by_status(&state.db, &request.destination_status_id, false)
                .await?,
        );
        tasks
    };
    let task_ids: Vec<String> = updated_tasks.iter().map(|task| task.id.clone()).collect();
    let tag_map = load_tag_map(state, project_id, &task_ids).await?;
    Ok(updated_tasks
        .into_iter()
        .map(|task| {
            task_to_dto(
                task.clone(),
                tag_map.get(&task.id).cloned().unwrap_or_default(),
            )
        })
        .collect())
}

async fn set_archived(state: &AppState, task_id: &str, archived: bool) -> ApiResult<TaskDto> {
    let task = TaskRepository::find_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| ApiError::not_found("task not found"))?;
    let mut active = TaskRepository::active_model_from(&task);
    active.archived_at = Set(if archived { Some(now_rfc3339()) } else { None });
    active.updated_at = Set(now_rfc3339());

    let event_type = if archived {
        "task.archived"
    } else {
        "task.restored"
    };
    let tx = state.db.begin().await?;
    let updated = TaskRepository::update(&tx, active).await?;
    write_activity_log(
        &tx,
        &task.project_id,
        Some(task_id),
        event_type,
        json!({ "taskId": task_id }),
    )
    .await?;
    tx.commit().await?;
    get_task(state, &updated.id).await
}

async fn validate_tag_ids(state: &AppState, project_id: &str, tag_ids: &[String]) -> ApiResult<()> {
    let tags = TagRepository::find_many_by_ids(&state.db, project_id, tag_ids).await?;
    if !TagRepository::ensure_all_ids_exist(&tags, tag_ids) {
        return Err(ApiError::validation(
            "tagIds must all belong to the project",
        ));
    }
    Ok(())
}

async fn load_tag_map(
    state: &AppState,
    project_id: &str,
    task_ids: &[String],
) -> ApiResult<HashMap<String, Vec<String>>> {
    let task_tags =
        TagRepository::list_task_tags_by_project(&state.db, project_id, Some(task_ids)).await?;
    Ok(TagRepository::build_tag_map(&task_tags))
}

fn validate_reorder_payload(
    moved_task: &task::Model,
    request: &ReorderTasksRequest,
    source_tasks: &[task::Model],
    destination_tasks: &[task::Model],
) -> ApiResult<()> {
    if moved_task.status_id != request.source_status_id {
        return Err(ApiError::conflict(
            "task is not in the declared source status",
        ));
    }

    let expected_ids: HashSet<String> = if request.source_status_id == request.destination_status_id
    {
        source_tasks.iter().map(|task| task.id.clone()).collect()
    } else {
        destination_tasks
            .iter()
            .map(|task| task.id.clone())
            .chain(std::iter::once(moved_task.id.clone()))
            .collect()
    };
    let received_ids: HashSet<String> = request.ordered_task_ids.iter().cloned().collect();
    if expected_ids != received_ids || request.ordered_task_ids.len() != expected_ids.len() {
        return Err(ApiError::validation(
            "orderedTaskIds must contain the final set of destination tasks exactly once",
        ));
    }
    Ok(())
}

async fn apply_destination_order<C: sea_orm::ConnectionTrait>(
    db: &C,
    destination_status_id: &str,
    ordered_task_ids: &[String],
    moved_task: &task::Model,
    destination_is_done: bool,
) -> ApiResult<()> {
    let now = now_rfc3339();
    let tasks = TaskRepository::list_by_ids(db, ordered_task_ids).await?;
    for (index, task_id) in ordered_task_ids.iter().enumerate() {
        let task = tasks
            .iter()
            .find(|task| task.id == *task_id)
            .ok_or_else(|| ApiError::not_found("task not found during reorder"))?;
        let mut active = TaskRepository::active_model_from(task);
        active.status_id = Set(destination_status_id.to_owned());
        active.position = Set((index as i64 + 1) * POSITION_STEP);
        if task.id == moved_task.id && moved_task.status_id != destination_status_id {
            active.completed_at = Set(if destination_is_done {
                Some(now.clone())
            } else {
                None
            });
        }
        active.updated_at = Set(now.clone());
        TaskRepository::update(db, active).await?;
    }
    Ok(())
}

async fn apply_source_order<C: sea_orm::ConnectionTrait>(
    db: &C,
    ordered_task_ids: &[String],
) -> ApiResult<()> {
    let now = now_rfc3339();
    let tasks = TaskRepository::list_by_ids(db, ordered_task_ids).await?;
    for (index, task_id) in ordered_task_ids.iter().enumerate() {
        let task = tasks
            .iter()
            .find(|task| task.id == *task_id)
            .ok_or_else(|| ApiError::not_found("task not found during reorder"))?;
        let mut active = TaskRepository::active_model_from(task);
        active.position = Set((index as i64 + 1) * POSITION_STEP);
        active.updated_at = Set(now.clone());
        TaskRepository::update(db, active).await?;
    }
    Ok(())
}
