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
    pub client_token: String,
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

    pub fn get_client_token() -> String {
        var("CLIENT_TOKEN").expect("missing client token as an env")
    }

    pub fn get_gemini_api_key() -> Option<String> {
        var("GEMINI_API_KEY").ok()
    }

    pub fn get_db_path() -> Option<String> {
        var("DB_PATH").ok()
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

#[derive(Debug, Clone, Default)]
pub struct EnvBuilder {
    pub client_token: Option<String>,
    pub gemini_api_key: Option<String>,
}

impl EnvBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_client_token(mut self, key: impl AsRef<OsStr>) -> Self {
        self.client_token = Some(var(key).expect("missing client token as an env"));
        self
    }

    pub fn with_gemini_api_key(mut self, key: impl AsRef<OsStr>) -> Self {
        self.gemini_api_key = var(key).ok();
        self
    }

    pub fn build(self) -> Env {
        Env {
            client_token: self.client_token.unwrap_or_else(Env::get_client_token),
            gemini_api_key: self.gemini_api_key.or_else(Env::get_gemini_api_key),
            db_path: Env::get_db_path(),
            log_level: Env::get_log_level(),
        }
    }
}
