use std::{fs, path::Path};

use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use toml::Value;

use crate::utils::{ConfigError, Merge};

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Owner {
    pub id: u64,
    pub url: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Info {
    pub prefix: String,
    pub owner: Owner,
    pub signature: Vec<String>,
    pub permissions: Vec<String>,
}

impl Default for Info {
    fn default() -> Self {
        toml::from_str(include_str!("../default/info.toml"))
            .expect("Critical Error: info.json is malformed!")
    }
}

impl Merge for Info {}

impl Info {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn signature_for(&self, username: &str) -> Option<String> {
        self.signature
            .choose(&mut rand::rng())
            .map(|s| s.replace("{username}", username))
    }

    pub fn from_file_over(self, path: &Path) -> Result<Info, ConfigError> {
        let mut default = Value::try_from(&self)?;
        let user_val: Value = toml::from_str(&fs::read_to_string(path)?)?;

        let user_owner = user_val.get("owner").cloned();

        Self::safe_merge(&mut default, user_val);

        if let (Some(owner), Some(table)) = (user_owner, default.as_table_mut()) {
            table.insert("owner".into(), owner);
        }

        Ok(default.try_into()?)
    }
}
