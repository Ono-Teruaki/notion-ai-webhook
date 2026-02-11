use axum::{extract::State, Json};
use chrono::{Duration, Local};
use reqwest::StatusCode;
use serde_json::json;

use crate::{
    api::{
        append_notion_block_to_page, create_page, delete_block, fetch_block_ids, fetch_notion_page,
        gen_notion_page_contents_from_gemini_api, query_database,
    },
    router::AppState,
    types::{
        ExtractText, GeminiAPIChatContent, GeminiAPIModel, GeminiAPIPrompt, GenerationConfig,
        NotionCreatePageRequest, NotionDatabaseQuery, NotionPageDetail, NotionWebhookPayload,
        Parent, Part, Role,
    },
};

pub async fn handle_weekly_report(
    State(state): State<AppState>,
    Json(payload): Json<NotionWebhookPayload>,
) -> StatusCode {
    tokio::spawn(async move {
        match weekly_report_process(&state, payload).await {
            Ok(_) => println!("Weekly report generated successfully"),
            Err(e) => println!("Weekly report generation failed: {}", e),
        }
    });

    StatusCode::OK
}

pub async fn weekly_report_process(
    state: &AppState,
    payload: NotionWebhookPayload,
) -> Result<(), Box<dyn std::error::Error>> {
    let report_page_id = &payload.data.id;

    // 1. Calculate Date Range (Last 7 days)
    let today = Local::now().date_naive();
    let one_week_ago = today - Duration::days(7);

    println!(
        "Generating weekly report for period: {} to {}",
        one_week_ago, today
    );

    // 2. Query Diary DB
    // Assumption: Property name for date is "Date" or "日付" or "Created time"
    // Trying "Date" first as it is most common for diaries.
    let filter = json!({
        "and": [
            {
                "property": "日付",
                "date": {
                    "on_or_after": one_week_ago.format("%Y-%m-%d").to_string()
                }
            },
            {
                "property": "日付",
                "date": {
                    "on_or_before": today.format("%Y-%m-%d").to_string()
                }
            }
        ]
    });

    let query = NotionDatabaseQuery {
        filter: Some(filter),
        sorts: Some(vec![json!({
            "property": "日付",
            "direction": "ascending"
        })]),
    };

    let diary_entries = query_database(
        &state.notion_service,
        &state.notion_service.diary_db_id,
        query,
    )
    .await?;

    println!("Found {} diary entries", diary_entries.results.len());

    // 3. Extract Content from Diary Entries
    let mut all_diary_text = String::new();

    for page in diary_entries.results {
        // Retrieve date from properties if possible for better context (Skipped for now to keep it simple, relying on content)
        // Fetch page blocks
        match fetch_notion_page(&state.notion_service, &page.id).await {
            Ok(page_detail) => {
                let page_text = extract_page_text(&page_detail);
                if !page_text.trim().is_empty() {
                    all_diary_text.push_str(&format!(
                        "\n--- Diary Entry ({}) ---\n{}\n",
                        page.id, page_text
                    ));
                }
            }
            Err(e) => {
                println!("Failed to fetch content for page {}: {}", page.id, e);
            }
        }
    }

    if all_diary_text.is_empty() {
        println!("No diary content found for the week.");
        // Clear existing content even if no diary found? Maybe just append "No content".
        // But the requirement is to clear content before update.
        clear_page_content(state, report_page_id).await?;

        append_notion_block_to_page(
            &state.notion_service,
            report_page_id,
            vec![crate::types::NotionBlock::paragraph(
                "対象期間の日記が見つかりませんでした。",
            )],
        )
        .await?;
        return Ok(());
    }

    // 4. Generate Prompt
    let prompt = gen_weekly_report_prompt(all_diary_text);

    // 5. Call Gemini
    let gened_blocks = gen_notion_page_contents_from_gemini_api(
        &state.gemini_service,
        prompt,
        GeminiAPIModel::Gemini3Flash,
    )
    .await?;

    // 6. Clear Existing Content & Append to Report Page (Webhook Source)
    println!("Clearing existing content in report page: {}", report_page_id);
    clear_page_content(state, report_page_id).await?;

    append_notion_block_to_page(&state.notion_service, report_page_id, gened_blocks.clone()) // Clone blocks for reuse
        .await?;

    // 7. Create New Page in Report DB
    let new_page_title = format!("週次レポート ({} ~ {})", one_week_ago, today);
    let create_page_request = NotionCreatePageRequest {
        parent: Parent {
            database_id: state.notion_service.report_db_id.clone(),
        },
        properties: json!({
            "名前": { // Assuming title property is "Name" or "名前"
                "title": [
                    {
                        "text": {
                            "content": new_page_title
                        }
                    }
                ]
            },
            "日付": {
                "date": {
                    "start": today.format("%Y-%m-%d").to_string()
                }
            }
        }),
        children: gened_blocks,
    };

    match create_page(&state.notion_service, create_page_request).await {
        Ok(page) => println!("Created new report page: {}", page.url),
        Err(e) => println!("Failed to create new report page: {}", e),
    }

    Ok(())
}

async fn clear_page_content(
    state: &AppState,
    page_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let blocks = fetch_block_ids(&state.notion_service, page_id).await?;
    for block in blocks {
        // Skip deleting child databases and buttons to avoid data loss/UI removal
        if block.block_type == "child_database" || block.block_type == "button" {
            println!(
                "Skipping deletion of {} block: {}",
                block.block_type, block.id
            );
            continue;
        }

        delete_block(&state.notion_service, &block.id).await?;
    }
    Ok(())
}

fn extract_page_text(page_detail: &NotionPageDetail) -> String {
    page_detail
        .body
        .results
        .iter()
        .filter_map(|b| b.extract_text())
        .collect::<Vec<_>>()
        .join("\n")
}

fn gen_weekly_report_prompt(diary_content: String) -> GeminiAPIPrompt {
    let system_instruction_str = include_str!("../prompts/weekly_report.txt").to_string();
    let mut system_instruction_parts = vec![];
    system_instruction_parts.push(Part {
        text: system_instruction_str,
    });

    let system_instruction = Some(GeminiAPIChatContent {
        role: Some(Role::User),
        parts: system_instruction_parts,
    });

    let contents = vec![GeminiAPIChatContent {
        role: Some(Role::User),
        parts: vec![Part {
            text: diary_content,
        }],
    }];

    let generation_config = Some(GenerationConfig::default());

    GeminiAPIPrompt {
        contents,
        system_instruction,
        generation_config,
    }
}