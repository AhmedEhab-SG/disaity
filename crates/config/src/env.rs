use std::{ffi::OsStr, str::FromStr};

use dotenv::var;
use strum::{Display, EnumString};

#[derive(Debug, Clone, Display, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct Env {
    pub client_token: Option<String>,
    pub gemini_api_key: Option<String>,
    pub db_path: Option<String>,
    pub log_level: Option<LogLevel>,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            client_token: Self::get_client_token(),
            gemini_api_key: Self::get_gemini_api_key(),
            db_path: Self::get_db_path(),
            log_level: Self::get_log_level(),
        }
    }
}

impl Env {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_client_token() -> Option<String> {
        var("CLIENT_TOKEN").ok()
    }

    fn get_gemini_api_key() -> Option<String> {
        var("GEMINI_API_KEY").ok()
    }

    fn get_db_path() -> Option<String> {
        var("DB_PATH").ok()
    }

    pub fn with_client_token(mut self, key: impl AsRef<OsStr>) -> Self {
        self.client_token = var(key).ok();
        self
    }

    pub fn with_gemini_api_key(mut self, key: impl AsRef<OsStr>) -> Self {
        self.gemini_api_key = var(key).ok();
        self
    }

    pub fn get_log_level() -> Option<LogLevel> {
        var("LOG_LEVEL").ok().map(|lvl| {
            LogLevel::from_str(lvl.as_str()).unwrap_or_else(|e| {
                eprintln!("LOG_LEVEL: {e}; falling back to {}", LogLevel::Debug);
                LogLevel::Debug
            })
        })
    }
}
