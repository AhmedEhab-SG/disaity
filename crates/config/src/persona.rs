use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::utils::Merge;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Persona {
    pub name: String,
    pub summary: String,
    pub personality: Personality,
    pub interactions: Interactions,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Personality {
    pub system: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Interactions {
    pub events: Events,
    pub colors: Colors,
    pub status: Vec<Status>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Events {
    pub lifecycle: Lifecycle,
    pub guild: Guild,
    pub dm: Dm,
}

type EventValues = Vec<String>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Lifecycle {
    pub on_ready: EventValues,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Guild {
    on_welcome: EventValues,
    on_join_vc: EventValues,
    on_leave_vc: EventValues,
    on_error: EventValues,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dm {
    on_error: EventValues,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Colors {
    pub primary: u32,
    pub success: u32,
    pub warning: u32,
    pub error: u32,
    pub accent: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Status {
    pub name: String,
    pub activity_type: ActivityType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityType {
    Playing,
    Listening,
    Watching,
    Competing,
    Streaming,
}

#[derive(Debug, Clone, Copy)]
pub enum Preset {
    Emilia,
    Rem,
}

// Error wrapper
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Toml(toml::de::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::Toml(e)
    }
}

impl Merge for Preset {}

impl Preset {
    pub fn raw(self) -> &'static str {
        match self {
            Self::Emilia => include_str!("../default/personas/emilia.toml"),
            Self::Rem => include_str!("../default/personas/rem.toml"),
        }
    }
}

impl Default for Persona {
    fn default() -> Self {
        Self::new(Preset::Emilia)
    }
}

impl Persona {
    pub fn new(base: Preset) -> Self {
        toml::from_str(base.raw()).expect("malformed default persona")
    }

    pub fn to_raw(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    pub fn from_default_over(self, path: &Path) -> Result<Self, ConfigError> {
        let mut default: toml::Value = self.to_raw().expect("malformed default persona").into();
        let user_value: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
        Preset::safe_merge(&mut default, user_value);
        Ok(default.try_into()?)
    }

    pub fn from_file_over(base: Preset, path: &Path) -> Result<Self, ConfigError> {
        let mut base_val: toml::Value = toml::from_str(base.raw())?;
        let user_val: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
        Preset::safe_merge(&mut base_val, user_val);
        Ok(base_val.try_into()?)
    }
}
