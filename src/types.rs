use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct NotionWebhookPayload {
    pub data: NotionPageData,
}

#[derive(Debug, Deserialize)]
pub struct NotionWebhookContent {
    pub content_type: AutomationContentType,
    pub payload: NotionWebhookPayload,
}

#[derive(Debug, Deserialize)]
pub enum AutomationContentType {
    Unknown,
    Diary,
}

#[derive(Debug, Deserialize)]
pub struct NotionPageData {
    id: String,
    url: String,
}
