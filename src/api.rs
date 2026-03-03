use crate::{
    service::{GeminiService, NotionService},
    types::*,
};
use reqwest::header::AUTHORIZATION;
use std::time::Duration;

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

    println!("Notion ページ追記結果: {:?}", response_body);

    Ok(())
}

pub async fn query_database(
    service: &NotionService,
    database_id: &str,
    query: NotionDatabaseQuery,
) -> Result<NotionDatabaseQueryResponse, Box<dyn std::error::Error>> {
    let url = format!("https://api.notion.com/v1/databases/{}/query", database_id);
    println!("データベースクエリ URL: {:?}", url);
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
        println!("データベースクエリ エラーステータス: {}", status);
        println!("データベースクエリ エラーボディ: {}", body_text);
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
        println!("ページ作成 エラーステータス: {}", status);
        println!("ページ作成 エラーボディ: {}", body_text);
        return Err(format!("Notion API Error: Status {}, Body: {}", status, body_text).into());
    }

    let response_data: NotionPage = serde_json::from_str(&body_text)?;
    Ok(response_data)
}

async fn push_to_gemini_api(
    service: &GeminiService,
    prompt: &GeminiAPIPrompt,
    model: GeminiAPIModel,
) -> Result<GeminiAPIResponse, Box<dyn std::error::Error>> {
    let model_name = model.model_name();

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model_name, service.api_key
    );

    const MAX_RETRIES: u32 = 3;
    let mut last_error: String = String::new();

    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            let wait_secs = 2u64.pow(attempt);
            println!(
                "Gemini API リトライ ({}/{}): model={}, {}秒待機中...",
                attempt,
                MAX_RETRIES - 1,
                model_name,
                wait_secs
            );
            tokio::time::sleep(Duration::from_secs(wait_secs)).await;
        }

        let response = match service.client.post(&url).json(prompt).send().await {
            Ok(r) => r,
            Err(e) => {
                last_error = format!(
                    "Gemini API リクエスト送信失敗: model={}, error={}",
                    model_name, e
                );
                println!("{}", last_error);
                continue;
            }
        };

        let status = response.status();
        println!(
            "Gemini API レスポンスステータス: model={}, status={}, attempt={}",
            model_name,
            status,
            attempt + 1
        );

        let body_bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                last_error = format!(
                    "Gemini API レスポンスボディの読み取りに失敗しました: model={}, error={}",
                    model_name, e
                );
                println!("{}", last_error);
                continue;
            }
        };

        let body_text = match String::from_utf8(body_bytes.to_vec()) {
            Ok(t) => t,
            Err(e) => {
                let preview: Vec<u8> = body_bytes.iter().take(200).cloned().collect();
                return Err(format!(
                    "Gemini API レスポンスが UTF-8 ではありません: model={}, error={}, bytes(先頭200)={:?}",
                    model_name, e, preview
                )
                .into());
            }
        };

        // 503 は高負荷による一時的なエラーのためリトライ
        if status.as_u16() == 503 {
            last_error = format!(
                "Gemini API エラー: model={}, status={}, body={}",
                model_name, status, body_text
            );
            println!("Gemini API 503 (高負荷、リトライします): model={}", model_name);
            continue;
        }

        if !status.is_success() {
            return Err(format!(
                "Gemini API エラー: model={}, status={}, body={}",
                model_name, status, body_text
            )
            .into());
        }

        let response_data: GeminiAPIResponse = serde_json::from_str(&body_text).map_err(|e| {
            format!(
                "Gemini レスポンスボディの JSON デコードに失敗しました: model={}, error={}, body={}",
                model_name, e, body_text
            )
        })?;

        return Ok(response_data);
    }

    Err(format!(
        "Gemini API 最大リトライ回数({})超過: model={}, last_error={}",
        MAX_RETRIES, model_name, last_error
    )
    .into())
}

pub async fn gen_notion_page_contents_from_gemini_api(
    service: &GeminiService,
    prompt: GeminiAPIPrompt,
    model: GeminiAPIModel,
) -> Result<Vec<NotionBlock>, Box<dyn std::error::Error>> {
    let response_data = if matches!(model, GeminiAPIModel::Gemini3Pro) {
        match push_to_gemini_api(service, &prompt, model)
            .await
            .map_err(|error| error.to_string())
        {
            Ok(response_data) => response_data,
            Err(primary_error_message) => {
                println!(
                    "Gemini Pro 呼び出し失敗。Flash モデルで再試行します: {}",
                    primary_error_message
                );
                push_to_gemini_api(service, &prompt, GeminiAPIModel::Gemini3Flash).await?
            }
        }
    } else {
        push_to_gemini_api(service, &prompt, model).await?
    };

    let generated_content_str = response_data
        .candidates
        .iter()
        .flat_map(|candidate| candidate.content.parts.iter())
        .find_map(|part| part.text.as_deref())
        .ok_or_else(|| {
            format!(
                "Gemini API レスポンスにテキストが含まれていません: {:?}",
                response_data
            )
        })?;
    println!("生成コンテンツ文字列: {:?}", generated_content_str);

    let generated_blocks: Vec<NotionBlock> = match serde_json::from_str(generated_content_str) {
        Ok(valid_blocks) => valid_blocks,
        Err(_) => vec![NotionBlock::heading_3("AIレスポンス生成に失敗しました")],
    };
    println!("生成ブロックリスト: {:?}", generated_blocks);

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
        println!("ブロック削除エラー: {}", response.status());
    }

    Ok(())
}
