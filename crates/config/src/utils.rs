use std::{error, fmt};

use toml::Value;

pub trait Merge {
    fn safe_merge(base: &mut Value, over: Value) {
        match (base, over) {
            (Value::Table(b), Value::Table(o)) => {
                for (k, v) in o {
                    match b.get_mut(&k) {
                        Some(bv) => Self::safe_merge(bv, v),
                        None => {
                            b.insert(k, v);
                        }
                    }
                }
            }

            (b, o) => *b = o,
        }
    }
}

// Error wrapper
pub enum ConfigError {
    Io(std::io::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
    SerdeJson(serde_json::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::TomlDe(e)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(e: toml::ser::Error) -> Self {
        Self::TomlSer(e)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerdeJson(e)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "can't read the config file: {e}"),
            Self::TomlDe(e) => write!(f, "{e}"),
            Self::TomlSer(e) => write!(f, "can't write the config as toml: {e}"),
            Self::SerdeJson(e) => write!(f, "JSON parse error: {e}"),
        }
    }
}

impl fmt::Debug for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl error::Error for ConfigError {}
