pub mod comments;
pub mod projects;
pub mod statuses;
pub mod system;
pub mod tags;
pub mod tasks;

use axum::{
    Router,
    extract::FromRequestParts,
    http::{HeaderValue, Request, header::HeaderName},
    middleware::{self, Next},
    response::Response,
    routing::{get, patch, post, put},
};
use sea_orm::DatabaseConnection;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

use crate::{app::AppState, dto::common::ApiMeta, error::ApiError};

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn into_meta(self) -> ApiMeta {
        ApiMeta::new(self.0)
    }
}

impl<S> FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<RequestId>()
            .cloned()
            .unwrap_or_else(|| RequestId(format!("req_{}", Uuid::now_v7()))))
    }
}

pub fn router(db: DatabaseConnection) -> Router {
    let state = AppState::new(db);
    Router::new()
        .route("/api/v1/health", get(system::health))
        .route("/api/v1/openapi.json", get(system::openapi))
        .route(
            "/api/v1/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        .route(
            "/api/v1/projects/{projectId}",
            get(projects::get_project)
                .patch(projects::update_project)
                .delete(projects::delete_project),
        )
        .route(
            "/api/v1/projects/{projectId}/board",
            get(tasks::get_board_snapshot),
        )
        .route(
            "/api/v1/projects/{projectId}/statuses",
            get(statuses::list_statuses).post(statuses::create_status),
        )
        .route(
            "/api/v1/projects/{projectId}/statuses/reorder",
            post(statuses::reorder_statuses),
        )
        .route(
            "/api/v1/statuses/{statusId}",
            patch(statuses::update_status).delete(statuses::delete_status),
        )
        .route(
            "/api/v1/projects/{projectId}/tasks",
            get(tasks::list_tasks).post(tasks::create_task),
        )
        .route(
            "/api/v1/projects/{projectId}/tasks/reorder",
            post(tasks::reorder_tasks),
        )
        .route(
            "/api/v1/tasks/{taskId}",
            get(tasks::get_task)
                .patch(tasks::update_task)
                .delete(tasks::delete_task),
        )
        .route("/api/v1/tasks/{taskId}/archive", post(tasks::archive_task))
        .route("/api/v1/tasks/{taskId}/restore", post(tasks::restore_task))
        .route(
            "/api/v1/tasks/{taskId}/comments",
            get(comments::list_comments).post(comments::create_comment),
        )
        .route(
            "/api/v1/comments/{commentId}",
            patch(comments::update_comment).delete(comments::delete_comment),
        )
        .route(
            "/api/v1/projects/{projectId}/tags",
            get(tags::list_tags).post(tags::create_tag),
        )
        .route(
            "/api/v1/tags/{tagId}",
            patch(tags::update_tag).delete(tags::delete_tag),
        )
        .route(
            "/api/v1/tasks/{taskId}/tags/{tagId}",
            put(tags::attach_tag).delete(tags::detach_tag),
        )
        .layer(middleware::from_fn(request_id_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn request_id_middleware(mut request: Request<axum::body::Body>, next: Next) -> Response {
    let header_name = HeaderName::from_static("x-request-id");
    let request_id = request
        .headers()
        .get(&header_name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("req_{}", Uuid::now_v7()));

    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));
    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(header_name, value);
    }
    response
}
