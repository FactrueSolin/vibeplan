use sea_orm::{ActiveValue::Set, TransactionTrait};
use serde_json::json;

use crate::{
    app::{AppState, comment_to_dto, ensure_non_blank, new_id, now_rfc3339, write_activity_log},
    dto::comment::{CommentDto, CreateCommentRequest, UpdateCommentRequest},
    entity::task_comment,
    error::{ApiError, ApiResult},
    repository::{comment::CommentRepository, task::TaskRepository},
};

pub async fn list_comments(state: &AppState, task_id: &str) -> ApiResult<Vec<CommentDto>> {
    TaskRepository::find_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| ApiError::not_found("task not found"))?;
    let comments = CommentRepository::list_by_task(&state.db, task_id).await?;
    Ok(comments.into_iter().map(comment_to_dto).collect())
}

pub async fn create_comment(
    state: &AppState,
    task_id: &str,
    request: CreateCommentRequest,
) -> ApiResult<CommentDto> {
    ensure_non_blank("authorName", &request.author_name)?;
    ensure_non_blank("content", &request.content)?;
    let task = TaskRepository::find_by_id(&state.db, task_id)
        .await?
        .ok_or_else(|| ApiError::not_found("task not found"))?;

    let now = now_rfc3339();
    let tx = state.db.begin().await?;
    let comment = CommentRepository::create(
        &tx,
        task_comment::ActiveModel {
            id: Set(new_id()),
            task_id: Set(task_id.to_owned()),
            author_name: Set(request.author_name.trim().to_owned()),
            content: Set(request.content.trim().to_owned()),
            created_at: Set(now.clone()),
            updated_at: Set(now.clone()),
        },
    )
    .await?;

    write_activity_log(
        &tx,
        &task.project_id,
        Some(task_id),
        "comment.created",
        json!({ "commentId": comment.id }),
    )
    .await?;
    tx.commit().await?;
    Ok(comment_to_dto(comment))
}

pub async fn update_comment(
    state: &AppState,
    comment_id: &str,
    request: UpdateCommentRequest,
) -> ApiResult<CommentDto> {
    ensure_non_blank("content", &request.content)?;
    let comment = CommentRepository::find_by_id(&state.db, comment_id)
        .await?
        .ok_or_else(|| ApiError::not_found("comment not found"))?;
    let task = TaskRepository::find_by_id(&state.db, &comment.task_id)
        .await?
        .ok_or_else(|| ApiError::not_found("task not found"))?;

    let mut active = CommentRepository::active_model_from(&comment);
    active.content = Set(request.content.trim().to_owned());
    active.updated_at = Set(now_rfc3339());

    let tx = state.db.begin().await?;
    let updated = CommentRepository::update(&tx, active).await?;
    write_activity_log(
        &tx,
        &task.project_id,
        Some(&task.id),
        "comment.updated",
        json!({ "commentId": updated.id }),
    )
    .await?;
    tx.commit().await?;

    Ok(comment_to_dto(updated))
}

pub async fn delete_comment(state: &AppState, comment_id: &str) -> ApiResult<()> {
    let comment = CommentRepository::find_by_id(&state.db, comment_id)
        .await?
        .ok_or_else(|| ApiError::not_found("comment not found"))?;
    let task = TaskRepository::find_by_id(&state.db, &comment.task_id)
        .await?
        .ok_or_else(|| ApiError::not_found("task not found"))?;

    let tx = state.db.begin().await?;
    let deleted = CommentRepository::delete(&tx, comment_id).await?;
    if deleted == 0 {
        return Err(ApiError::not_found("comment not found"));
    }
    write_activity_log(
        &tx,
        &task.project_id,
        Some(&task.id),
        "comment.deleted",
        json!({ "commentId": comment_id }),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}
