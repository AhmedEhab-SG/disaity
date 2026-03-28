use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Owner {
    pub username: String,
    pub id: u64,
    pub url: String,
    pub icon_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Permission {
    pub send_message: String,
    pub admin: String,
    pub voice_join: String,
}

#[derive(Deserialize, Debug)]
pub struct InfoRegistry {
    pub prefix: String,
    pub client_id: u64,
    pub owner: Owner,
    pub permission: Permission,
    pub invite_ul: String,
    pub author_icon: String,
}

impl Default for InfoRegistry {
    fn default() -> Self {
        let json_data = include_str!("../../config/info.json");
        serde_json::from_str(json_data).expect("Critical Error: info.json is malformed!")
    }
}

impl InfoRegistry {
    pub fn new() -> Self {
        Self::default()
    }
}
