use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Owner {
    pub username: String,
    pub id: u64,
    pub url: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct InfoRegistry {
    pub prefix: String,
    pub owner: Owner,
    pub permissions: Vec<String>,
}

impl Default for InfoRegistry {
    fn default() -> Self {
        toml::from_str(include_str!("../default/info.toml"))
            .expect("Critical Error: info.json is malformed!")
    }
}

impl InfoRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}
