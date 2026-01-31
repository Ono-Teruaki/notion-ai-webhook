use axum::http::header::HeaderMap;

use crate::{
    api::fetch_notion_page,
    types::{
        AutomationContentType, ExtractText, GeminiAPIPrompt, GeminiAPIPromptContent,
        GenerationConfig, NotionPageDetail, NotionWebhookContent, NotionWebhookPayload, Part, Role,
    },
};

pub async fn handle_webhook(headers: HeaderMap, body: String) -> String {
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
            return "Invalid data".to_string();
        }
    };
    println!("webhook payload: {:#?}", payload);
    let webhook_content = NotionWebhookContent {
        content_type: content_type,
        payload: payload,
    };

    let page_data = match fetch_notion_page(webhook_content.payload).await {
        Ok(data) => data,
        Err(e) => {
            println!("Error: {}", e);
            return "Failed to get notion page data".to_string();
        }
    };

    println!("Notion API Response Data: {page_data:#?}");

    body
}

fn gen_diary_prompt(page_detail: NotionPageDetail) -> GeminiAPIPrompt {
    let system_instruction = Some(String::from(
        r#"
あなたは日記のレビュー結果をNotion API形式で出力する専用マシンです。
日記の内容を汲み取り今後の方針や疑問解消などをしながらポジティブ気味にフィードバック文を考えてください。

【重要ルール】

    出力は必ず [ で始まり ] で終わる有効なJSON配列のみとしてください。

    JSON以外の説明、挨拶、Markdownタグ（```jsonなど）は一切含めないでください。

    内容は heading_2（総評）と paragraph（詳細レビュー）で構成してください。

    Notion APIの "Append block children" 形式に従ってください。

【出力テンプレート】 [ { "object": "block", "type": "heading_2", "heading_2": { "rich_text": [{ "type": "text", "text": { "content": "ここに総評" } }] } }, { "object": "block", "type": "paragraph", "paragraph": { "rich_text": [{ "type": "text", "text": { "content": "ここに詳細" } }] } } ]
"#,
    ));

    let page_contents: Vec<Part> = page_detail
        .body
        .results
        .iter()
        .filter_map(|b| b.extract_text())
        .map(|text| Part { text })
        .collect();

    let mut contents = vec![];
    contents.push(GeminiAPIPromptContent {
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
