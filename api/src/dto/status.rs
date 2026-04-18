use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::dto::common::ApiMeta;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatusDto {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub color: String,
    pub sort_order: i64,
    pub is_done: bool,
    pub is_hidden: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: String,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateStatusRequest {
    #[validate(length(min = 1, max = 120))]
    pub name: String,
    #[validate(length(min = 4, max = 32))]
    pub color: String,
    pub is_done: Option<bool>,
    pub is_hidden: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusRequest {
    #[validate(length(min = 1, max = 120))]
    pub name: Option<String>,
    #[validate(length(min = 4, max = 32))]
    pub color: Option<String>,
    pub is_done: Option<bool>,
    pub is_hidden: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReorderStatusesRequest {
    pub ordered_status_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StatusResponse {
    pub data: StatusDto,
    pub meta: ApiMeta,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StatusListResponse {
    pub data: Vec<StatusDto>,
    pub meta: ApiMeta,
}
