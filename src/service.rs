use std::env;

use dotenv::dotenv;
use reqwest::Client;

#[derive(Clone)]
pub struct NotionService {
    pub client: Client,
    pub api_key: String,
}

impl NotionService {
    pub fn new(client: Client) -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();
        let api_key = env::var("NOTION_API_KEY")?;
        Ok(Self { client, api_key })
    }
}

#[derive(Clone)]
pub struct GeminiService {
    pub client: Client,
    pub api_key: String,
}

impl GeminiService {
    pub fn new(client: Client) -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();
        let api_key = env::var("GEMINI_API_KEY")?;
        Ok(Self { client, api_key })
    }
}
