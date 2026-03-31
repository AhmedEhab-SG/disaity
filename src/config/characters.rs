use serde::Deserialize;
use std::collections::HashMap;
use strum::Display;

#[derive(Display, Deserialize, Debug, Clone, Eq, Hash, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Character {
    Emilia,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CharacterConfig {
    pub name: String,
    pub personality: String,
}

#[derive(Deserialize, Debug)]
pub struct CharacterRegistry {
    #[serde(flatten)]
    pub characters: HashMap<Character, CharacterConfig>,
}

impl Default for CharacterRegistry {
    fn default() -> Self {
        let yaml_data = include_str!("../../config/characters.yaml");
        serde_yaml::from_str(yaml_data).expect("Critical Error: character.yaml is malformed!")
    }
}

impl CharacterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_character(&self, key: &Character) -> &CharacterConfig {
        self.characters
            .get(key)
            .unwrap_or_else(|| panic!("Expect character '{key}', but it wasn't found in registry"))
    }
}
