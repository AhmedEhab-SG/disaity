use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Steam,
    Epic,
    Gog,
    Ubisoft,
    Ea,
    BattleNet,
    Unknown,
}

impl Platform {
    pub fn from_platforms(platforms: &str) -> Self {
        let platforms = platforms.to_lowercase();

        if platforms.contains("steam") {
            Self::Steam
        } else if platforms.contains("epic") {
            Self::Epic
        } else if platforms.contains("gog") {
            Self::Gog
        } else if platforms.contains("ubisoft") {
            Self::Ubisoft
        } else if platforms.contains("origin") || platforms.contains("ea app") {
            Self::Ea
        } else if platforms.contains("battle.net") {
            Self::BattleNet
        } else {
            Self::Unknown
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Assets {
    pub provider: HashMap<Provider, String>,
    pub platform: HashMap<Platform, String>,
}

impl Default for Assets {
    fn default() -> Self {
        toml::from_str(include_str!("../default/assets.toml"))
            .expect("Critical Error: assets.toml is malformed!")
    }
}

impl Assets {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_logo(&self, p: Provider) -> String {
        self.provider.get(&p).cloned().unwrap_or_default()
    }

    pub fn get_platform_logo(&self, p: Platform) -> String {
        self.platform.get(&p).cloned().unwrap_or_default()
    }
}
