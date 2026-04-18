use serde_json::Value;
use utoipa::OpenApi;

use crate::{
    dto::{
        comment::{
            CommentDto, CommentListResponse, CommentResponse, CreateCommentRequest,
            UpdateCommentRequest,
        },
        common::ApiMeta,
        project::{
            CreateProjectRequest, ProjectDto, ProjectListResponse, ProjectResponse,
            UpdateProjectRequest,
        },
        status::{
            CreateStatusRequest, ReorderStatusesRequest, StatusDto, StatusListResponse,
            StatusResponse, UpdateStatusRequest,
        },
        tag::{
            CreateTagRequest, TagDto, TagListResponse, TagResponse, TaskTagDto, UpdateTagRequest,
        },
        task::{
            ArchivedFilter, BoardResponse, BoardSnapshotDto, BoardSummaryDto, CreateTaskRequest,
            ReorderTasksRequest, SortOrder, TaskDto, TaskListResponse, TaskResponse, TaskSortBy,
            UpdateTaskRequest,
        },
    },
    entity::sea_orm_active_enums::TaskPriority,
    error::ErrorEnvelope,
    routes,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Plan API",
        version = "v1",
        description = "Local kanban task management API"
    ),
    servers(
        (url = "http://localhost:3001/api/v1", description = "Local development server")
    ),
    tags(
        (name = "Projects", description = "Project management"),
        (name = "Statuses", description = "Board statuses"),
        (name = "Tasks", description = "Task and board operations"),
        (name = "Comments", description = "Task comments"),
        (name = "Tags", description = "Project tags"),
        (name = "System", description = "System endpoints")
    ),
    paths(
        routes::system::health,
        routes::system::openapi,
        routes::projects::list_projects,
        routes::projects::create_project,
        routes::projects::get_project,
        routes::projects::update_project,
        routes::projects::delete_project,
        routes::statuses::list_statuses,
        routes::statuses::create_status,
        routes::statuses::update_status,
        routes::statuses::delete_status,
        routes::statuses::reorder_statuses,
        routes::tasks::get_board_snapshot,
        routes::tasks::list_tasks,
        routes::tasks::create_task,
        routes::tasks::get_task,
        routes::tasks::update_task,
        routes::tasks::delete_task,
        routes::tasks::archive_task,
        routes::tasks::restore_task,
        routes::tasks::reorder_tasks,
        routes::comments::list_comments,
        routes::comments::create_comment,
        routes::comments::update_comment,
        routes::comments::delete_comment,
        routes::tags::list_tags,
        routes::tags::create_tag,
        routes::tags::update_tag,
        routes::tags::delete_tag,
        routes::tags::attach_tag,
        routes::tags::detach_tag
    ),
    components(
        schemas(
            ApiMeta,
            ErrorEnvelope,
            TaskPriority,
            ProjectDto,
            ProjectResponse,
            ProjectListResponse,
            CreateProjectRequest,
            UpdateProjectRequest,
            StatusDto,
            StatusResponse,
            StatusListResponse,
            CreateStatusRequest,
            UpdateStatusRequest,
            ReorderStatusesRequest,
            TagDto,
            TaskTagDto,
            TagResponse,
            TagListResponse,
            CreateTagRequest,
            UpdateTagRequest,
            CommentDto,
            CommentResponse,
            CommentListResponse,
            CreateCommentRequest,
            UpdateCommentRequest,
            TaskDto,
            TaskResponse,
            TaskListResponse,
            BoardSummaryDto,
            BoardSnapshotDto,
            BoardResponse,
            CreateTaskRequest,
            UpdateTaskRequest,
            ReorderTasksRequest,
            ArchivedFilter,
            TaskSortBy,
            SortOrder
        )
    )
)]
pub struct ApiDoc;

pub fn openapi_json_value() -> Value {
    serde_json::to_value(ApiDoc::openapi()).expect("openapi serialization should succeed")
}
