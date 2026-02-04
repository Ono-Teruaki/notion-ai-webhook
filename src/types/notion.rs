use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct NotionWebhookPayload {
    pub data: NotionPageRef,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotionPageRef {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotionPageDetail {
    pub body: NotionBlockResponse,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotionAppendBlockRequest {
    pub children: Vec<NotionBlock>,
    pub position: AppendPositionType,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppendPositionType {
    Start,
    End,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotionBlockResponse {
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
    BulletedListItem {
        bulleted_list_item: BlockContent,
    },
    NumberedListItem {
        numbered_list_item: BlockContent,
    },
    ToDo {
        to_do: ToDoBlockContent, // ToDoは「チェック状態」を持つため別構造体
    },
    Toggle {
        toggle: ToggleBlockContent,
    },
    Quote {
        quote: BlockContent,
    },
    Callout {
        callout: BlockContent, // 本来はアイコンも持てますが、テキストのみなら共通化可能
    },
    Divider {
        divider: EmptyStruct, // 区切り線は内容が空のオブジェクト {}
    },
    Code {
        code: CodeBlockContent,
    },
    #[serde(other)]
    Unsupported,
}

impl NotionBlock {
    pub fn paragraph(text: &str) -> Self {
        Self::Paragraph {
            paragraph: BlockContent::new(text),
        }
    }
    pub fn heading_1(text: &str) -> Self {
        Self::Heading1 {
            heading_1: BlockContent::new(text),
        }
    }
    pub fn heading_2(text: &str) -> Self {
        Self::Heading2 {
            heading_2: BlockContent::new(text),
        }
    }
    pub fn heading_3(text: &str) -> Self {
        Self::Heading3 {
            heading_3: BlockContent::new(text),
        }
    }
    pub fn code(text: &str, language: String) -> Self {
        Self::Code {
            code: CodeBlockContent::new(text, language),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockContent {
    pub rich_text: Vec<NotionRichText>,
}

impl BlockContent {
    fn new(text: &str) -> Self {
        Self {
            rich_text: vec![NotionRichText::new(text)],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CodeBlockContent {
    // caption
    rich_text: Vec<NotionRichText>,
    language: String,
}

impl CodeBlockContent {
    fn new(text: &str, language: String) -> Self {
        Self {
            rich_text: vec![NotionRichText::new(text)],
            language,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToggleBlockContent {
    pub rich_text: Vec<NotionRichText>,
    // トグルの中身を再帰的に持てるようにする
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<NotionBlock>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotionRichText {
    Text {
        text: NotionTextContent,
        #[serde(skip_serializing)]
        plain_text: Option<String>,
        #[serde(default)]
        annotations: RichTextAnnotations,
    },
    Mention {
        mention: serde_json::Value,
        #[serde(default)]
        annotations: RichTextAnnotations,
        #[serde(skip_serializing)]
        plain_text: Option<String>,
    },
    Equation {
        equation: serde_json::Value,
        #[serde(default)]
        annotations: RichTextAnnotations,
        #[serde(skip_serializing)]
        plain_text: Option<String>,
    },
}

impl NotionRichText {
    fn new(text: &str) -> Self {
        Self::Text {
            text: NotionTextContent::new(text),
            plain_text: None,
            annotations: RichTextAnnotations::default(),
        }
    }

    fn plain_text(&self) -> Option<&str> {
        match self {
            NotionRichText::Text { plain_text, .. } => plain_text.as_deref(),
            NotionRichText::Mention { plain_text, .. } => plain_text.as_deref(),
            NotionRichText::Equation { plain_text, .. } => plain_text.as_deref(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct RichTextAnnotations {
    bold: bool,
    italic: bool,
    strikethrough: bool,
    underline: bool,
    code: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<RichTextColor>,
}

impl Default for RichTextAnnotations {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
            code: false,
            color: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RichTextColor {
    Default,
    Red,
    Blue,
    Pink,
    Green,
    Yellow,
    Purple,
    Glay,
    Blown,
    Orange,
}

// テキスト抽出用トレイト
pub trait ExtractText {
    fn extract_text(&self) -> Option<String>;
}

impl<T: HasRichText> ExtractText for T {
    fn extract_text(&self) -> Option<String> {
        let rich_text = self.get_rich_text();
        if rich_text.is_empty() {
            return None;
        }
        Some(
            rich_text
                .iter()
                .filter_map(|t| t.plain_text())
                .map(|t| t)
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
            NotionBlock::BulletedListItem { bulleted_list_item } => {
                bulleted_list_item.extract_text()
            }
            NotionBlock::NumberedListItem { numbered_list_item } => {
                numbered_list_item.extract_text()
            }
            NotionBlock::ToDo { to_do } => to_do.extract_text(),
            NotionBlock::Toggle { toggle } => toggle.extract_text(),
            NotionBlock::Quote { quote } => quote.extract_text(),
            NotionBlock::Callout { callout } => callout.extract_text(),
            NotionBlock::Divider { .. } => None,
            NotionBlock::Code { code } => code.extract_text(),
            NotionBlock::Unsupported => None,
        }
    }
}

pub trait HasRichText {
    fn get_rich_text(&self) -> &[NotionRichText];
}

impl HasRichText for BlockContent {
    fn get_rich_text(&self) -> &[NotionRichText] {
        &self.rich_text
    }
}

impl HasRichText for ToDoBlockContent {
    fn get_rich_text(&self) -> &[NotionRichText] {
        &self.rich_text
    }
}

impl HasRichText for ToggleBlockContent {
    fn get_rich_text(&self) -> &[NotionRichText] {
        &self.rich_text
    }
}

impl HasRichText for CodeBlockContent {
    fn get_rich_text(&self) -> &[NotionRichText] {
        &self.rich_text
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToDoBlockContent {
    pub rich_text: Vec<NotionRichText>,
    pub checked: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EmptyStruct {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct NotionTextContent {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<NotionLinkText>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct NotionLinkText {
    pub url: String,
}

impl NotionTextContent {
    fn new(text: &str) -> Self {
        Self {
            content: text.to_string(),
            link: None,
        }
    }
}
