use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::dto::common::ApiMeta;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TagDto {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub color: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: String,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskTagDto {
    pub project_id: String,
    pub task_id: String,
    pub tag_id: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagRequest {
    #[validate(length(min = 1, max = 60))]
    pub name: String,
    #[validate(length(min = 4, max = 32))]
    pub color: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTagRequest {
    #[validate(length(min = 1, max = 60))]
    pub name: Option<String>,
    #[validate(length(min = 4, max = 32))]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TagResponse {
    pub data: TagDto,
    pub meta: ApiMeta,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TagListResponse {
    pub data: Vec<TagDto>,
    pub meta: ApiMeta,
}
