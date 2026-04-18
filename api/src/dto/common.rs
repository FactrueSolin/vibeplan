use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiMeta {
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

impl ApiMeta {
    pub fn new(request_id: String) -> Self {
        Self {
            request_id,
            page: None,
            page_size: None,
            total: None,
        }
    }

    pub fn with_page(mut self, page: u64, page_size: u64, total: u64) -> Self {
        self.page = Some(page);
        self.page_size = Some(page_size);
        self.total = Some(total);
        self
    }
}
