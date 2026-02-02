use axum::http::header::HeaderMap;
use reqwest::StatusCode;

use crate::{
    api::{fetch_notion_page, gen_notion_page_contents_from_gemini_api},
    types::{
        AutomationContentType, ExtractText, GeminiAPIChatContent, GeminiAPIPrompt,
        GenerationConfig, NotionPageDetail, NotionWebhookPayload, Part, Role,
    },
};

pub async fn handle_webhook(headers: HeaderMap, body: String) -> StatusCode {
    let content_type_str = headers
        .get("content_type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let content_type = match content_type_str {
        "diary" => AutomationContentType::Diary,
        _ => AutomationContentType::Unknown,
    };

    let payload: NotionWebhookPayload = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => {
            println!("Failed to JSON perse: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    tokio::spawn(async move {
        match process_automation(payload, content_type).await {
            Ok(_) => println!("Automation completed successfully"),
            Err(e) => println!("Automation failed: {}", e),
        }
    });

    StatusCode::OK
}

pub async fn process_automation(
    payload: NotionWebhookPayload,
    content_type: AutomationContentType,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    println!("Webhook payload: {:?}", payload);
    let notion_page_content = fetch_notion_page(&client, payload).await?;
    println!("Notion Page Content: {:?}", notion_page_content);

    let prompt = match content_type {
        AutomationContentType::Diary => gen_diary_prompt(notion_page_content),
        AutomationContentType::Unknown => return Err("Error: Unknown Content Type".into()),
    };

    let response = gen_notion_page_contents_from_gemini_api(&client, prompt).await?;

    println!("Gemini API Response: {response:?}");

    Ok(())
}

fn gen_diary_prompt(page_detail: NotionPageDetail) -> GeminiAPIPrompt {
    let system_instruction_str = r#"
あなたは日記のレビュー結果をNotion API (Append block children) 形式のJSON配列で出力する専門マシンです。

【重要ルール】
1. 出力は必ず [ で始まり ] で終わる有効なJSON配列のみ。
2. Markdownの解説、挨拶、```json などの囲みは一切禁止。
3. 以下のJSON構造を遵守してください。箇条書きや、トグルなどの使用は任せますが、必ずAppend block childrenのレスポンス形式にしてください。

【出力スキーマ】
[
   {
      "object": "block",
      "type": "heading_2",
      "heading_2": {
        "rich_text": [
          {
            "type": "text",
            "text": { "content": "見出し(AIレビューなど)" }
          }
        ]
      }
    },
    {
      "object": "block",
      "type": "paragraph",
      "paragraph": {
        "rich_text": [
          {
            "type": "text",
            "text": { "content": "ここに詳細" }
          }
        ]
      }
    }
]
"#.to_string();
    let mut system_instruction_parts = vec![];
    system_instruction_parts.push(Part {
        text: system_instruction_str,
    });

    let system_instruction = Some(GeminiAPIChatContent {
        role: Some(Role::User),
        parts: system_instruction_parts,
    });

    let page_contents: Vec<Part> = page_detail
        .body
        .results
        .iter()
        .filter_map(|b| b.extract_text())
        .map(|text| Part { text })
        .collect();

    let mut contents = vec![];
    contents.push(GeminiAPIChatContent {
        role: Some(Role::User),
        parts: page_contents,
    });

    let generation_config = Some(GenerationConfig::default());

    GeminiAPIPrompt {
        contents,
        system_instruction,
        generation_config,
    }
}
