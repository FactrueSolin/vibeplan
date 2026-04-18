pub mod comment;
pub mod project;
pub mod status;
pub mod tag;
pub mod task;

use sea_orm::{ActiveValue::Set, DatabaseConnection};
use serde_json::Value;
use time::{Date, OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use crate::{
    dto::{
        comment::CommentDto,
        project::ProjectDto,
        status::StatusDto,
        tag::{TagDto, TaskTagDto},
        task::TaskDto,
    },
    entity::{
        activity_log, project as entity_project, tag as entity_tag, task as entity_task,
        task_comment, task_status, task_tag,
    },
    error::{ApiError, ApiResult},
    repository::activity_log::ActivityLogRepository,
};

pub const POSITION_STEP: i64 = 1000;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

pub fn new_id() -> String {
    Uuid::now_v7().to_string()
}

pub fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("rfc3339 formatting should not fail")
}

pub fn normalize_slug(slug: &str) -> ApiResult<String> {
    let mut output = String::new();
    let mut last_dash = false;

    for ch in slug.trim().chars() {
        let normalized = ch.to_ascii_lowercase();
        if normalized.is_ascii_alphanumeric() {
            output.push(normalized);
            last_dash = false;
        } else if !last_dash {
            output.push('-');
            last_dash = true;
        }
    }

    let output = output.trim_matches('-').to_owned();
    if output.is_empty() {
        return Err(ApiError::validation("slug must contain letters or numbers"));
    }

    Ok(output)
}

pub fn ensure_non_blank(field: &str, value: &str) -> ApiResult<()> {
    if value.trim().is_empty() {
        return Err(ApiError::validation(format!("{field} must not be empty")));
    }
    Ok(())
}

pub fn validate_optional_date(date: &Option<String>, field: &str) -> ApiResult<()> {
    if let Some(value) = date {
        Date::parse(
            value,
            &time::macros::format_description!("[year]-[month]-[day]"),
        )
        .map_err(|_| ApiError::validation(format!("{field} must be a valid YYYY-MM-DD date")))?;
    }
    Ok(())
}

pub fn validate_date_range(
    start_date: &Option<String>,
    due_date: &Option<String>,
) -> ApiResult<()> {
    validate_optional_date(start_date, "startDate")?;
    validate_optional_date(due_date, "dueDate")?;
    if let (Some(start_date), Some(due_date)) = (start_date, due_date) {
        let start = Date::parse(
            start_date,
            &time::macros::format_description!("[year]-[month]-[day]"),
        )
        .map_err(|_| ApiError::validation("startDate must be a valid YYYY-MM-DD date"))?;
        let due = Date::parse(
            due_date,
            &time::macros::format_description!("[year]-[month]-[day]"),
        )
        .map_err(|_| ApiError::validation("dueDate must be a valid YYYY-MM-DD date"))?;
        if start > due {
            return Err(ApiError::validation(
                "dueDate must be greater than or equal to startDate",
            ));
        }
    }
    Ok(())
}

pub async fn write_activity_log<C: sea_orm::ConnectionTrait>(
    db: &C,
    project_id: &str,
    task_id: Option<&str>,
    event_type: &str,
    payload: Value,
) -> ApiResult<()> {
    ActivityLogRepository::create(
        db,
        activity_log::ActiveModel {
            id: Set(new_id()),
            project_id: Set(project_id.to_owned()),
            task_id: Set(task_id.map(str::to_owned)),
            event_type: Set(event_type.to_owned()),
            payload_json: Set(payload.to_string()),
            created_at: Set(now_rfc3339()),
        },
    )
    .await?;
    Ok(())
}

pub fn project_to_dto(model: entity_project::Model) -> ProjectDto {
    ProjectDto {
        id: model.id,
        name: model.name,
        slug: model.slug,
        description: model.description,
        color: model.color,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

pub fn status_to_dto(model: task_status::Model) -> StatusDto {
    StatusDto {
        id: model.id,
        project_id: model.project_id,
        name: model.name,
        color: model.color,
        sort_order: model.sort_order,
        is_done: model.is_done,
        is_hidden: model.is_hidden,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

pub fn task_to_dto(model: entity_task::Model, tag_ids: Vec<String>) -> TaskDto {
    TaskDto {
        id: model.id,
        project_id: model.project_id,
        status_id: model.status_id,
        title: model.title,
        description: model.description,
        priority: model.priority,
        position: model.position,
        start_date: model.start_date,
        due_date: model.due_date,
        completed_at: model.completed_at,
        archived_at: model.archived_at,
        tag_ids,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

pub fn comment_to_dto(model: task_comment::Model) -> CommentDto {
    CommentDto {
        id: model.id,
        task_id: model.task_id,
        author_name: model.author_name,
        content: model.content,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

pub fn tag_to_dto(model: entity_tag::Model) -> TagDto {
    TagDto {
        id: model.id,
        project_id: model.project_id,
        name: model.name,
        color: model.color,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

pub fn task_tag_to_dto(model: task_tag::Model) -> TaskTagDto {
    TaskTagDto {
        project_id: model.project_id,
        task_id: model.task_id,
        tag_id: model.tag_id,
        created_at: model.created_at,
    }
}
