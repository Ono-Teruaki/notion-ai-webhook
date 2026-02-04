use serde::{Deserialize, Serialize};

pub enum GeminiAPIModel {
    Gemini3Flash,
    Gemini3Pro,
}

impl GeminiAPIModel {
    pub fn model_name(&self) -> &'static str {
        match &self {
            Self::Gemini3Flash => "gemini-3-flash-preview",
            Self::Gemini3Pro => "gemini-3-pro-preview",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiAPIPrompt {
    pub contents: Vec<GeminiAPIChatContent>,
    pub system_instruction: Option<GeminiAPIChatContent>,
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiAPIChatContent {
    pub role: Option<Role>,
    pub parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    User,
    Model,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Part {
    pub text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    pub temperature: f32,
    pub response_mime_type: ResponseMimeType,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            response_mime_type: ResponseMimeType::Json,
        }
    }
}

#[derive(Debug, Serialize, Default)]
pub enum ResponseMimeType {
    #[default]
    #[serde(rename = "text/plain")]
    Text,
    #[serde(rename = "application/json")]
    Json,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiAPIResponse {
    pub candidates: Vec<GeminiAPICandidate>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiAPICandidate {
    pub content: GeminiAPIChatContent,
}
