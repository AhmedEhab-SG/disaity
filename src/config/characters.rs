use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Character {
    pub name: String,
    pub personality: String,
}

#[derive(Deserialize, Debug)]
pub struct CharacterRegistry {
    #[serde(flatten)]
    pub characters: HashMap<String, Character>,
}

impl Default for CharacterRegistry {
    fn default() -> Self {
        let ymal_data = include_str!("../../config/characters.yaml");
        serde_yaml::from_str(ymal_data).expect("Critical Error: character.yaml is malformed!")
    }
}

impl CharacterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_character(&self, key: &str) -> Option<&Character> {
        self.characters.get(key)
    }
}
