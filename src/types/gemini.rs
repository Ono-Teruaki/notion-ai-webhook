use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiAPIPrompt {
    pub contents: Vec<GeminiAPIPromptContent>,
    pub system_instruction: Option<String>,
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Serialize)]
pub struct GeminiAPIPromptContent {
    pub role: Option<Role>,
    pub parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Model,
}

#[derive(Debug, Serialize)]
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
