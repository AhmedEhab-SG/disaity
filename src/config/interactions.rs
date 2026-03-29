use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ProviderLogUrls {
    pub youtube: String,
    pub soundcloud: String,
    pub spotify: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Colors {
    pub help: u32,
    pub error: u32,
    pub default: u32,
    pub blurple: u32,
    pub action: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Errors {
    pub general: Vec<String>,
    pub direct: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Messages {
    pub join: Vec<String>,
    pub leave: Vec<String>,
    pub built: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Status {
    pub name: String,
    pub activity_type: u8,
}

#[derive(Deserialize, Debug)]
pub struct InteractionsRegistry {
    pub ready: String,
    pub status: Vec<Status>,
    pub messages: Messages,
    pub errors: Errors,
    pub colors: Colors,
    pub provider_logo_urls: ProviderLogUrls,
}

impl Default for InteractionsRegistry {
    fn default() -> Self {
        let json_data = include_str!("../../config/interactions.json");
        serde_json::from_str(json_data).expect("Critical Error: interactions.json is malformed!")
    }
}

impl InteractionsRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}
