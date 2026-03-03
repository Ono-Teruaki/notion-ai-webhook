use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum GeminiAPIModel {
    Gemini3Flash,
    Gemini3Pro,
}

impl GeminiAPIModel {
    pub fn model_name(&self) -> &'static str {
        match &self {
            Self::Gemini3Flash => "gemini-3-flash-preview",
            Self::Gemini3Pro => "gemini-3.1-pro-preview",
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
    #[serde(default)]
    pub candidates: Vec<GeminiAPICandidate>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeminiAPICandidate {
    #[serde(default)]
    pub content: GeminiAPIResponseContent,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeminiAPIResponseContent {
    #[serde(default)]
    pub parts: Vec<GeminiAPIResponsePart>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct GeminiAPIResponsePart {
    #[serde(default)]
    pub text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_prompt_serialization() {
        let prompt = GeminiAPIPrompt {
            contents: vec![GeminiAPIChatContent {
                role: Some(Role::User),
                parts: vec![Part { text: "Hello".to_string() }],
            }],
            system_instruction: Some(GeminiAPIChatContent {
                role: Some(Role::User),
                parts: vec![Part { text: "Be helpful".to_string() }],
            }),
            generation_config: Some(GenerationConfig::default()),
        };

        let json = serde_json::to_string(&prompt).unwrap();
        assert!(json.contains("\"contents\""));
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"systemInstruction\""));
        assert!(json.contains("\"responseMimeType\":\"application/json\""));
    }

    #[test]
    fn test_gemini_response_without_candidates_is_valid() {
        let json = r#"{"promptFeedback":{"blockReason":"SAFETY"}}"#;
        let response: GeminiAPIResponse = serde_json::from_str(json).unwrap();
        assert!(response.candidates.is_empty());
    }

    #[test]
    fn test_gemini_response_part_without_text_is_valid() {
        let json = r#"{"candidates":[{"content":{"parts":[{"inlineData":{"mimeType":"text/plain","data":"abc"}}]}}]}"#;
        let response: GeminiAPIResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.candidates.len(), 1);
        assert_eq!(response.candidates[0].content.parts.len(), 1);
        assert!(response.candidates[0].content.parts[0].text.is_none());
    }
}
