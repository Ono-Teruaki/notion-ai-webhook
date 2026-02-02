use crate::types::*;
use dotenv::dotenv;
use reqwest::header::AUTHORIZATION;
use std::env;

pub async fn fetch_notion_page(
    client: &reqwest::Client,
    webhook_payload: NotionWebhookPayload,
) -> Result<NotionPageDetail, Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("NOTION_API_KEY")?;
    let page_id = &webhook_payload.data.id;
    let url = format!("https://api.notion.com/v1/blocks/{}/children", page_id);

    let response = client
        .get(&url)
        .header("Notion-Version", "2025-09-03")
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .send()
        .await?
        .json::<NotionBlockResponse>()
        .await?;

    let page_detail = NotionPageDetail {
        page_ref: webhook_payload.data,
        body: response,
    };

    Ok(page_detail)
}

// pub async fn push_to_notion_page(

// )

async fn push_to_gemini_api(
    client: &reqwest::Client,
    prompt: GeminiAPIPrompt,
) -> Result<GeminiAPIResponse, Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("GEMINI_API_KEY")?;
    let model = "gemini-3-flash-preview";

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let response = client
        .post(url)
        .json(&prompt)
        .send()
        .await?
        .json::<GeminiAPIResponse>()
        .await?;

    Ok(response)
}

pub async fn gen_notion_page_contents_from_gemini_api(
    client: &reqwest::Client,
    prompt: GeminiAPIPrompt,
) -> Result<Vec<NotionAppendBlock>, Box<dyn std::error::Error>> {
    let response_data = push_to_gemini_api(client, prompt).await?;
    let generated_content_str = &response_data.candidates[0].content.parts[0].text;
    println!("Generated Content String: {:?}", generated_content_str);

    let generated_blocks: Vec<NotionAppendBlock> = serde_json::from_str(&generated_content_str)?;
    println!("Generated Block List: {:#?}", generated_blocks);

    Ok(generated_blocks)
}
