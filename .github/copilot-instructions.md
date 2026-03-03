# Copilot Instructions

## Build & Run

```bash
cargo build                  # debug build
cargo build --release        # release build (used in Docker)
cargo test                   # run all tests
cargo test <test_name>       # run a single test (e.g. `cargo test test_gen_diary_prompt`)
```

Requires a `.env` file with:
```
NOTION_API_KEY=...
GEMINI_API_KEY=...
NOTION_DIARY_DB_ID=...
NOTION_REPORT_DB_ID=...
PORT=8080  # optional, defaults to 8080
```

## Architecture

This is an Axum HTTP server that bridges **Notion webhooks → Gemini AI → Notion API**.

**Request flow for each automation:**
1. Notion triggers a POST to `/webhook/{diary|diary-weekly-report|review}` with `{ data: { id: "<page_id>" } }`
2. The handler immediately spawns a background task via `tokio::spawn` and returns `200 OK`
3. The background task fetches Notion page blocks, builds a Gemini prompt, calls Gemini, and appends the AI-generated blocks back to the Notion page

**Key modules:**
- `src/router.rs` — Axum routes and `AppState` (holds `NotionService` + `GeminiService`)
- `src/api.rs` — all raw HTTP calls to Notion API and Gemini API
- `src/automation/` — one file per webhook type (`diary.rs`, `review.rs`, `weekly_report.rs`)
- `src/types/notion.rs` — Notion block types + serde models
- `src/types/gemini.rs` — Gemini API request/response types
- `src/prompts/` — system prompt text files embedded at compile time via `include_str!`

**Weekly report specifics:** Queries the diary DB for entries in the last 7 days filtered by the `日付` Notion property, aggregates their text, generates a report, clears the trigger page, appends new content, then creates a new page in the report DB.

## Key Conventions

**Notion API versioning:** Two different versions are in use — `2025-09-03` for block read/append, `2022-06-28` for database queries and page creation. Don't unify them without testing.

**`NotionBlock` construction:** Always use the constructor methods (`NotionBlock::paragraph()`, `::heading_1()`, `::heading_2()`, `::heading_3()`, `::code()`) rather than building the enum variants directly.

**`NotionRichText.plain_text`** is `#[serde(skip_serializing)]` — it is read-only (populated by Notion's API) and must not be sent in requests. Use the `ExtractText` trait to extract text from blocks.

**Gemini output must be valid `Vec<NotionBlock>` JSON.** The prompts instruct Gemini to output JSON. If `serde_json::from_str` fails on the response, `gen_notion_page_contents_from_gemini_api` falls back to a single `heading_3("AIレスポンス生成に失敗しました")` block.

**Gemini model fallback:** `GeminiAPIModel::Gemini3Pro` automatically retries with `Gemini3Flash` on failure. Use `Gemini3Flash` directly when Pro fallback is undesirable.

**Service structs** (`NotionService`, `GeminiService`) are thin credential holders passed via Axum `State`. All API calls are free functions in `src/api.rs` that accept a service reference.

**Prompt files** live in `src/prompts/` and are embedded at compile time with `include_str!`. Adding a new automation requires a corresponding `.txt` prompt file.

**`clear_page_content`** skips blocks of type `child_database`, `button`, and `unsupported` to avoid data loss when resetting a page.

## Deployment

Multi-stage Docker build → deployed to Google Cloud Run on port 8080.
