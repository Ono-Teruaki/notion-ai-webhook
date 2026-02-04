use reqwest::Client;

#[derive(Clone)]
pub struct NotionService {
    pub client: Client,
    pub api_key: String,
}

impl NotionService {
    pub fn new(client: Client, api_key: String) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { client, api_key })
    }
}

#[derive(Clone)]
pub struct GeminiService {
    pub client: Client,
    pub api_key: String,
}

impl GeminiService {
    pub fn new(client: Client, api_key: String) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { client, api_key })
    }
}
