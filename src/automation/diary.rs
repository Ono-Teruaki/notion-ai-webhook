use axum::{extract::State, Json};
use reqwest::StatusCode;

use crate::{
    api::{
        append_notion_block_to_page, fetch_notion_page, gen_notion_page_contents_from_gemini_api,
    },
    router::AppState,
    types::{
        ExtractText, GeminiAPIChatContent, GeminiAPIModel, GeminiAPIPrompt, GenerationConfig,
        NotionPageDetail, NotionWebhookPayload, Part, Role,
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

    let gened_block_contents = gen_notion_page_contents_from_gemini_api(
        &state.gemini_service,
        prompt,
        GeminiAPIModel::Gemini3Flash,
    )
    .await?;

    println!("Gemini API Response: {gened_block_contents:?}");

    append_notion_block_to_page(&state.notion_service, page_id, gened_block_contents).await?;

    Ok(())
}

fn gen_diary_prompt(page_detail: NotionPageDetail) -> GeminiAPIPrompt {
    let system_instruction_str = include_str!("../prompts/diary_review.txt").to_string();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NotionBlockResponse, NotionBlock};

    #[test]
    fn test_gen_diary_prompt() {
        let page_detail = NotionPageDetail {
            body: NotionBlockResponse {
                results: vec![
                    NotionBlock::paragraph("Today was a good day."),
                    NotionBlock::heading_2("Goals"),
                ],
            },
        };

        let prompt = gen_diary_prompt(page_detail);

        assert_eq!(prompt.contents.len(), 1);
        assert_eq!(prompt.contents[0].parts.len(), 2);
        assert_eq!(prompt.contents[0].parts[0].text, "Today was a good day.");
        assert_eq!(prompt.contents[0].parts[1].text, "Goals");
        assert!(prompt.system_instruction.is_some());
    }
}
