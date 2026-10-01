use serde::Deserialize;

use super::card::Effect;

/// A `data/characters/*.json` file.
#[derive(Debug, Clone, Deserialize)]
pub struct CharacterFile {
    pub class: String,
    pub name: String,
    pub img: String,
    pub cards: Vec<String>,
    pub stats: Stats,
    #[serde(default)]
    pub modifiers: Vec<Effect>,
    pub triggers: Triggers,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Stats {
    pub hp: u32,
    pub max_energy: u32,
    pub initiative: i32,
}

/// Same shape as a card's effects, applied on the matching trigger instead
/// of on a target.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Triggers {
    #[serde(default)]
    pub start: Vec<Effect>,
    #[serde(default)]
    pub end: Vec<Effect>,
    #[serde(default)]
    pub play: Vec<Effect>,
}
