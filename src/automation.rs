use axum::{extract::State, Json};
use reqwest::StatusCode;

use crate::{
    api::{
        append_notion_block_to_page, fetch_notion_page, gen_notion_page_contents_from_gemini_api,
    },
    router::AppState,
    types::{
        ExtractText, GeminiAPIChatContent, GeminiAPIPrompt, GenerationConfig, NotionPageDetail,
        NotionWebhookPayload, Part, Role,
    },
};

pub async fn handle_diary_automation(
    State(state): State<AppState>,
    Json(payload): Json<NotionWebhookPayload>,
) -> StatusCode {
    tokio::spawn(async move {
        match diary_automation_process(&state, payload).await {
            Ok(_) => println!("Automation completed successfully"),
            Err(e) => println!("Automation failed: {}", e),
        }
    });

    StatusCode::OK
}

pub async fn diary_automation_process(
    state: &AppState,
    payload: NotionWebhookPayload,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Webhook payload: {:?}", payload);
    let page_id = &payload.data.id;
    let notion_page_content = fetch_notion_page(&state.notion_service, page_id).await?;
    println!("Notion Page Content: {:?}", notion_page_content);

    let prompt = gen_diary_prompt(notion_page_content);

    let gened_block_contents =
        gen_notion_page_contents_from_gemini_api(&state.gemini_service, prompt).await?;

    println!("Gemini API Response: {gened_block_contents:?}");

    append_notion_block_to_page(&state.notion_service, page_id, gened_block_contents).await?;

    Ok(())
}

fn gen_diary_prompt(page_detail: NotionPageDetail) -> GeminiAPIPrompt {
    let system_instruction_str = r#"
あなたは、ユーザーの日記を読み解き、Notion形式でフィードバックを生成するAIメンターです。
日記の内容を整理しながら、悩みや疑問の解消、問題への対処法などのフィードバックをしてください。

【重要ルール】
1. 出力は必ず [ で始まり ] で終わる有効なJSON配列のみ。
2. Markdownの解説、挨拶、```json などの囲みは一切禁止。
3. 以下のJSON出力スキーマ例に従ってください。箇条書きや、トグルなどの使用は任せますが、必ずNotion APIのAppend block childrenのレスポンス形式にしてください。

【出力スキーマ例】
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
