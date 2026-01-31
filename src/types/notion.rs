use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct NotionWebhookContent {
    pub content_type: AutomationContentType,
    pub payload: NotionWebhookPayload,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotionWebhookPayload {
    pub data: NotionPageRef,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotionPageRef {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AutomationContentType {
    Diary,
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotionPageDetail {
    pub page_ref: NotionPageRef,
    pub body: NotionBlockList,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotionBlockList {
    pub results: Vec<NotionBlock>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotionBlock {
    #[serde(rename = "heading_1")]
    Heading1 {
        heading_1: BlockContent,
    },
    #[serde(rename = "heading_2")]
    Heading2 {
        heading_2: BlockContent,
    },
    #[serde(rename = "heading_3")]
    Heading3 {
        heading_3: BlockContent,
    },
    Paragraph {
        paragraph: BlockContent,
    },
    #[serde(other)]
    Unsupported,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockContent {
    pub rich_text: Vec<PlainText>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlainText {
    pub plain_text: String,
}

// テキスト抽出用トレイト
pub trait ExtractText {
    fn extract_text(&self) -> Option<String>;
}

impl ExtractText for BlockContent {
    fn extract_text(&self) -> Option<String> {
        if self.rich_text.is_empty() {
            return None;
        }
        Some(
            self.rich_text
                .iter()
                .map(|t| t.plain_text.as_str())
                .collect::<Vec<_>>()
                .join(""),
        )
    }
}

impl ExtractText for NotionBlock {
    fn extract_text(&self) -> Option<String> {
        match self {
            NotionBlock::Heading1 { heading_1 } => heading_1.extract_text(),
            NotionBlock::Heading2 { heading_2 } => heading_2.extract_text(),
            NotionBlock::Heading3 { heading_3 } => heading_3.extract_text(),
            NotionBlock::Paragraph { paragraph } => paragraph.extract_text(),
            NotionBlock::Unsupported => None,
        }
    }
}
