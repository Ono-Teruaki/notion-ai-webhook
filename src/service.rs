use reqwest::Client;

#[derive(Clone)]
pub struct NotionService {
    pub client: Client,
    pub api_key: String,
    pub diary_db_id: String,
    pub report_db_id: String,
}

impl NotionService {
    pub fn new(
        client: Client,
        api_key: String,
        diary_db_id: String,
        report_db_id: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            client,
            api_key: api_key.trim().to_string(),
            diary_db_id: diary_db_id.trim().to_string(),
            report_db_id: report_db_id.trim().to_string(),
        })
    }
}

#[derive(Clone)]
pub struct GeminiService {
    pub client: Client,
    pub api_key: String,
}

impl GeminiService {
    pub fn new(client: Client, api_key: String) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            client,
            api_key: api_key.trim().to_string(),
        })
    }
}