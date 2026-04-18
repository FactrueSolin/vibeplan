use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
    dto::{
        common::ApiMeta,
        project::ProjectDto,
        status::StatusDto,
        tag::{TagDto, TaskTagDto},
    },
    entity::sea_orm_active_enums::TaskPriority,
};

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskDto {
    pub id: String,
    pub project_id: String,
    pub status_id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
    pub position: i64,
    #[schema(value_type = String, format = Date)]
    pub start_date: Option<String>,
    #[schema(value_type = String, format = Date)]
    pub due_date: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub completed_at: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub archived_at: Option<String>,
    pub tag_ids: Vec<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: String,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardSummaryDto {
    pub active_task_count: u64,
    pub done_task_count: u64,
    pub archived_task_count: u64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardSnapshotDto {
    pub project: ProjectDto,
    pub statuses: Vec<StatusDto>,
    pub tasks: Vec<TaskDto>,
    pub tags: Vec<TagDto>,
    pub task_tags: Vec<TaskTagDto>,
    pub summary: BoardSummaryDto,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    pub status_id: String,
    #[validate(length(min = 1, max = 240))]
    pub title: String,
    #[validate(length(max = 12000))]
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub tag_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskRequest {
    #[validate(length(min = 1, max = 240))]
    pub title: Option<String>,
    #[validate(length(max = 12000))]
    #[serde(default)]
    pub description: Option<Option<String>>,
    pub priority: Option<Option<TaskPriority>>,
    pub status_id: Option<String>,
    #[serde(default)]
    pub start_date: Option<Option<String>>,
    #[serde(default)]
    pub due_date: Option<Option<String>>,
    #[serde(default)]
    pub tag_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReorderTasksRequest {
    pub moved_task_id: String,
    pub source_status_id: String,
    pub destination_status_id: String,
    pub ordered_task_ids: Vec<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct BoardQuery {
    pub include_archived: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ArchivedFilter {
    Exclude,
    Only,
    Include,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TaskSortBy {
    UpdatedAt,
    DueDate,
    CreatedAt,
    Position,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct TaskListQuery {
    pub q: Option<String>,
    pub status_id: Option<String>,
    pub priority: Option<TaskPriority>,
    pub tag_id: Option<String>,
    pub archived: Option<ArchivedFilter>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<TaskSortBy>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TaskResponse {
    pub data: TaskDto,
    pub meta: ApiMeta,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TaskListResponse {
    pub data: Vec<TaskDto>,
    pub meta: ApiMeta,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BoardResponse {
    pub data: BoardSnapshotDto,
    pub meta: ApiMeta,
}
