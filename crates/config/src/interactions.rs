use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    YouTube,
    SoundCloud,
    Spotify,
    Unknown,
}

impl Provider {
    pub fn from_url(url: &str) -> Self {
        if url.contains("youtube.com") || url.contains("youtu.be") {
            Self::YouTube
        } else if url.contains("soundcloud.com") {
            Self::SoundCloud
        } else if url.contains("spotify.com") {
            Self::Spotify
        } else {
            Self::Unknown
        }
    }
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
    pub provider_logo_urls: HashMap<Provider, String>,
}

impl Default for InteractionsRegistry {
    fn default() -> Self {
        let json_data = include_str!("../default/interactions.json");
        serde_json::from_str(json_data).expect("Critical Error: interactions.json is malformed!")
    }
}

impl InteractionsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_logo(&self, provider: Provider) -> String {
        self.provider_logo_urls
            .get(&provider)
            .cloned()
            .unwrap_or_else(|| "".to_string())
    }
}
