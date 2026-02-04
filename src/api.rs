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
