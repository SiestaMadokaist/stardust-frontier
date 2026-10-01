use serde::Deserialize;

/// A `data/cards/*.json` file: a namespaced list of cards.
#[derive(Debug, Clone, Deserialize)]
pub struct CardFile {
    pub namespace: String,
    #[serde(default)]
    pub active: bool,
    pub root: CardRoot,
    pub cards: Vec<CardDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CardRoot {
    pub images: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CardDef {
    #[serde(default)]
    pub origins: Vec<String>,
    pub name: String,
    pub src: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub description: String,
    pub cost: Vec<Cost>,
    #[serde(default)]
    pub targets: Vec<Target>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Cost {
    #[serde(rename = "type")]
    pub kind: CostType,
    pub value: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostType {
    Energy,
    Hp,
    #[serde(rename = "hand:select")]
    HandSelect,
    #[serde(rename = "hand:random")]
    HandRandom,
    #[serde(rename = "deck:top")]
    DeckTop,
    #[serde(rename = "deck:bottom")]
    DeckBottom,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Target {
    pub side: Side,
    pub multi: Multi,
    pub select: bool,
    pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Ally,
    Enemy,
}

/// Either a fixed target count, or `"all"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Multi {
    Count(u32),
    All,
}

impl<'de> Deserialize<'de> for Multi {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Num(u32),
            Str(String),
        }

        match Raw::deserialize(deserializer)? {
            Raw::Num(n) => Ok(Multi::Count(n)),
            Raw::Str(s) if s == "all" => Ok(Multi::All),
            Raw::Str(s) => Err(serde::de::Error::custom(format!(
                "invalid `multi` value: {s:?} (expected a number or \"all\")"
            ))),
        }
    }
}

/// An effect applied to a target (or a modifier/trigger entry on a character).
/// `kind` is left open-ended (plain string) since new effect types are added
/// as gameplay features land, rather than kept as a closed enum here.
#[derive(Debug, Clone, Deserialize)]
pub struct Effect {
    #[serde(rename = "type")]
    pub kind: String,
    pub value: i32,
}
