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
        .json::<NotionBlockList>()
        .await?;

    let page_detail = NotionPageDetail {
        page_ref: webhook_payload.data,
        body: response,
    };

    Ok(page_detail)
}
