pub mod card;
pub mod character;

use std::collections::HashMap;

use bevy::prelude::Resource;
use card::CardDef;
use character::CharacterFile;

/// All loaded cards, keyed by their id (`{namespace}:{name}`).
#[derive(Debug, Default, Resource)]
pub struct CardDatabase {
    pub cards: HashMap<String, CardDef>,
}

/// All loaded characters, keyed by `name`.
#[derive(Debug, Default, Resource)]
pub struct CharacterDatabase {
    pub characters: HashMap<String, CharacterFile>,
}
