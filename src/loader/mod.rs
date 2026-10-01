use std::{
    fs,
    path::{Path, PathBuf},
};

use bevy::prelude::*;
use serde::Deserialize;

use crate::data::{card::CardFile, character::CharacterFile, CardDatabase, CharacterDatabase};

/// A `./loader/**/*.json` manifest: `root` is resolved relative to the game
/// root (not the manifest file's own location), so mods keep their data
/// under their own folder while only dropping a manifest into `./loader`.
#[derive(Debug, Clone, Deserialize)]
struct Manifest {
    root: PathBuf,
    entries: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestEntry {
    #[serde(rename = "type")]
    kind: EntryType,
    paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EntryType {
    Cards,
    Character,
}

pub struct DataLoaderPlugin;

impl Plugin for DataLoaderPlugin {
    fn build(&self, app: &mut App) {
        let (cards, characters) = load_all(Path::new("./loader"));
        app.insert_resource(cards).insert_resource(characters);
    }
}

fn load_all(loader_dir: &Path) -> (CardDatabase, CharacterDatabase) {
    let mut cards = CardDatabase::default();
    let mut characters = CharacterDatabase::default();

    for manifest_path in find_json_files(loader_dir) {
        let manifest = match read_json::<Manifest>(&manifest_path) {
            Ok(m) => m,
            Err(err) => {
                error!("failed to parse manifest {}: {err}", manifest_path.display());
                continue;
            }
        };

        for entry in &manifest.entries {
            for rel_path in &entry.paths {
                let full_path = manifest.root.join(rel_path);
                match entry.kind {
                    EntryType::Cards => load_card_file(&full_path, &mut cards),
                    EntryType::Character => load_character_file(&full_path, &mut characters),
                }
            }
        }
    }

    (cards, characters)
}

fn find_json_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        warn!("loader directory {} not found", dir.display());
        return out;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(find_json_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "json") {
            out.push(path);
        }
    }

    out
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn load_card_file(path: &Path, db: &mut CardDatabase) {
    match read_json::<CardFile>(path) {
        Ok(file) => {
            if !file.active {
                return;
            }
            for card in file.cards {
                // `card.name` is already fully-qualified as `{namespace}:{name}`
                // (characters reference cards by this same string).
                db.cards.insert(card.name.clone(), card);
            }
        }
        Err(err) => error!("failed to load card file {}: {err}", path.display()),
    }
}

fn load_character_file(path: &Path, db: &mut CharacterDatabase) {
    match read_json::<CharacterFile>(path) {
        Ok(file) => {
            db.characters.insert(file.name.clone(), file);
        }
        Err(err) => error!("failed to load character file {}: {err}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_manifests_under_loader_dir() {
        let (cards, characters) = load_all(Path::new("./loader"));

        assert!(characters.characters.contains_key("wizard"));
        assert!(characters.characters.contains_key("healer"));

        assert!(cards.cards.contains_key("example:heal"));
        assert!(cards.cards.contains_key("example:firebolt"));
        assert!(cards.cards.contains_key("example:meteor"));
    }
}
