use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::dto::common::ApiMeta;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDto {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: String,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    #[validate(length(min = 1, max = 120))]
    pub name: String,
    #[validate(length(min = 1, max = 120))]
    pub slug: String,
    #[validate(length(max = 4000))]
    pub description: Option<String>,
    #[validate(length(min = 4, max = 32))]
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    #[validate(length(min = 1, max = 120))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 120))]
    pub slug: Option<String>,
    #[validate(length(max = 4000))]
    #[serde(default)]
    pub description: Option<Option<String>>,
    #[validate(length(min = 4, max = 32))]
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct PaginationQuery {
    #[param(default = 1, minimum = 1)]
    pub page: Option<u64>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProjectResponse {
    pub data: ProjectDto,
    pub meta: ApiMeta,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProjectListResponse {
    pub data: Vec<ProjectDto>,
    pub meta: ApiMeta,
}
