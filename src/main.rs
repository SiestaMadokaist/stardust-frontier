mod data;
mod loader;

use bevy::prelude::*;
use data::{CardDatabase, CharacterDatabase};
use loader::DataLoaderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DataLoaderPlugin)
        .add_systems(Startup, log_loaded_data)
        .run();
}

fn log_loaded_data(cards: Res<CardDatabase>, characters: Res<CharacterDatabase>) {
    info!(
        "loaded {} card(s), {} character(s)",
        cards.cards.len(),
        characters.characters.len()
    );
    for id in cards.cards.keys() {
        info!("  card: {id}");
    }
    for name in characters.characters.keys() {
        info!("  character: {name}");
    }
}
