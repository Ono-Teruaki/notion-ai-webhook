use crate::{
    service::{GeminiService, NotionService},
    types::*,
};
use reqwest::header::AUTHORIZATION;

pub async fn fetch_notion_page(
    service: &NotionService,
    page_id: &str,
) -> Result<NotionPageDetail, Box<dyn std::error::Error>> {
    let url = format!("https://api.notion.com/v1/blocks/{}/children", page_id);

    let response = service
        .client
        .get(&url)
        .header("Notion-Version", "2025-09-03")
        .header(AUTHORIZATION, format!("Bearer {}", service.api_key))
        .send()
        .await?
        .json::<NotionBlockResponse>()
        .await?;

    let page_detail = NotionPageDetail { body: response };

    Ok(page_detail)
}

pub async fn append_notion_block_to_page(
    service: &NotionService,
    page_id: &str,
    block_contents: Vec<NotionBlock>,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://api.notion.com/v1/blocks/{}/children", page_id);
    let request_data = NotionAppendBlockRequest {
        children: block_contents,
        position: AppendPositionType::End,
    };

    let response = service
        .client
        .patch(&url)
        .header("Notion-Version", "2025-09-03")
        .header(AUTHORIZATION, format!("Bearer {}", service.api_key))
        .json(&request_data)
        .send()
        .await?;

    let response_body = response.text().await;

    println!("Notion page Append Result: {:?}", response_body);

    Ok(())
}

pub async fn query_database(
    service: &NotionService,
    database_id: &str,
    query: NotionDatabaseQuery,
) -> Result<NotionDatabaseQueryResponse, Box<dyn std::error::Error>> {
    let url = format!("https://api.notion.com/v1/databases/{}/query", database_id);
    println!("Query Database URL: {:?}", url);
    let response = service
        .client
        .post(&url)
        .header("Notion-Version", "2022-06-28")
        .header(AUTHORIZATION, format!("Bearer {}", service.api_key))
        .json(&query)
        .send()
        .await?;

    let status = response.status();
    let body_text = response.text().await?;

    if !status.is_success() {
        println!("Query Database Error Status: {}", status);
        println!("Query Database Error Body: {}", body_text);
        return Err(format!("Notion API Error: Status {}, Body: {}", status, body_text).into());
    }

    let response_data: NotionDatabaseQueryResponse = serde_json::from_str(&body_text)?;
    Ok(response_data)
}

pub async fn create_page(
    service: &NotionService,
    request: NotionCreatePageRequest,
) -> Result<NotionPage, Box<dyn std::error::Error>> {
    let url = "https://api.notion.com/v1/pages";
    let response = service
        .client
        .post(url)
        .header("Notion-Version", "2022-06-28")
        .header(AUTHORIZATION, format!("Bearer {}", service.api_key))
        .json(&request)
        .send()
        .await?;

    let status = response.status();
    let body_text = response.text().await?;

    if !status.is_success() {
        println!("Create Page Error Status: {}", status);
        println!("Create Page Error Body: {}", body_text);
        return Err(format!("Notion API Error: Status {}, Body: {}", status, body_text).into());
    }

    let response_data: NotionPage = serde_json::from_str(&body_text)?;
    Ok(response_data)
}

async fn push_to_gemini_api(
    service: &GeminiService,
    prompt: GeminiAPIPrompt,
    model: GeminiAPIModel,
) -> Result<GeminiAPIResponse, Box<dyn std::error::Error>> {
    let model = model.model_name();

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, service.api_key
    );

    let response = service
        .client
        .post(url)
        .json(&prompt)
        .send()
        .await?
        .json::<GeminiAPIResponse>()
        .await?;

    Ok(response)
}

pub async fn gen_notion_page_contents_from_gemini_api(
    service: &GeminiService,
    prompt: GeminiAPIPrompt,
    model: GeminiAPIModel,
) -> Result<Vec<NotionBlock>, Box<dyn std::error::Error>> {
    let response_data = push_to_gemini_api(service, prompt, model).await?;
    let generated_content_str = &response_data.candidates[0].content.parts[0].text;
    println!("Generated Content String: {:?}", generated_content_str);

    let generated_blocks: Vec<NotionBlock> = match serde_json::from_str(&generated_content_str) {
        Ok(valid_blocks) => valid_blocks,
        Err(_) => vec![NotionBlock::heading_3("AIレスポンス生成に失敗しました")],
    };
    println!("Generated Block List: {:?}", generated_blocks);

    Ok(generated_blocks)
}

pub async fn fetch_block_ids(
    service: &NotionService,
    page_id: &str,
) -> Result<Vec<NotionBlockId>, Box<dyn std::error::Error>> {
    let url = format!("https://api.notion.com/v1/blocks/{}/children", page_id);

    let response = service
        .client
        .get(&url)
        .header("Notion-Version", "2022-06-28")
        .header(AUTHORIZATION, format!("Bearer {}", service.api_key))
        .send()
        .await?
        .json::<NotionBlockIdListResponse>()
        .await?;

    Ok(response.results)
}

pub async fn delete_block(
    service: &NotionService,
    block_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://api.notion.com/v1/blocks/{}", block_id);
    let response = service
        .client
        .delete(&url)
        .header("Notion-Version", "2022-06-28")
        .header(AUTHORIZATION, format!("Bearer {}", service.api_key))
        .send()
        .await?;

    if !response.status().is_success() {
        println!("Delete Block Error: {}", response.status());
    }

    Ok(())
}