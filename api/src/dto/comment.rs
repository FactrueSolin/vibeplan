use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::dto::common::ApiMeta;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommentDto {
    pub id: String,
    pub task_id: String,
    pub author_name: String,
    pub content: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: String,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommentRequest {
    #[validate(length(min = 1, max = 120))]
    pub author_name: String,
    #[validate(length(min = 1, max = 8000))]
    pub content: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCommentRequest {
    #[validate(length(min = 1, max = 8000))]
    pub content: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CommentResponse {
    pub data: CommentDto,
    pub meta: ApiMeta,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CommentListResponse {
    pub data: Vec<CommentDto>,
    pub meta: ApiMeta,
}
