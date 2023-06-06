use std::{fmt::Display, path::PathBuf, sync::OnceLock};

use directories::ProjectDirs;

use iced::widget::image;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::serde_help::*;

const CDRAGON_URL: &str = "https://raw.communitydragon.org/latest/game/";

static DIR: OnceLock<ProjectDirs> = OnceLock::new();
static CACHE_DIR: OnceLock<PathBuf> = OnceLock::new();
static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

trait ImageHandleDefault {
    fn default() -> Self;
}

impl ImageHandleDefault for image::Handle {
    fn default() -> Self {
        image::Handle::from_path(CACHE_DIR.get().unwrap().join("tft_item_unknown.png"))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Handle {
    pub handle: image::Handle,
    pub url: String,
}

impl Default for Handle {
    fn default() -> Self {
        Self {
            handle: ImageHandleDefault::default(),
            url: Default::default(),
        }
    }
}

impl Serialize for Handle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.url)
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    api_name: String,
    associated_traits: Vec<String>,
    composition: Vec<String>,
    #[serde(deserialize_with = "deserialize_null_default")]
    desc: String,
    effects: Value,
    from: Option<Value>, // always None
    #[serde(deserialize_with = "deserialize_image")]
    icon: Handle,
    id: Option<Value>, // always None
    incompatible_traits: Vec<String>,
    #[serde(deserialize_with = "deserialize_null_default")]
    name: String,
    unique: bool,
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct ItemsDisplay(Vec<Item>);

impl Display for ItemsDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return write!(f, "no items");
        }
        let mut comma_separated = String::new();

        for item in &self.0[0..self.0.len() - 1] {
            comma_separated.push_str(&item.to_string());
            comma_separated.push_str(", ");
        }

        comma_separated.push_str(&self.0[self.0.len() - 1].to_string());
        write!(f, "{}", comma_separated)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    #[serde(deserialize_with = "deserialize_null_default")]
    name: String,
    #[serde(deserialize_with = "deserialize_null_default")]
    value: Vec<f64>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ability {
    #[serde(deserialize_with = "deserialize_null_default")]
    desc: String,
    #[serde(deserialize_with = "deserialize_image")]
    icon: Handle,
    #[serde(deserialize_with = "deserialize_null_default")]
    name: String,
    variables: Vec<Variable>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    armor: Option<f64>,
    attack_speed: Option<f64>,
    crit_chance: Option<f64>,
    crit_multiplier: f64,
    damage: Option<f64>,
    hp: Option<f64>,
    initial_mana: f64,
    magic_resist: Option<f64>,
    mana: f64,
    range: f64,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Champion {
    ability: Ability,
    api_name: String,
    cost: u8,
    #[serde(deserialize_with = "deserialize_image")]
    square_icon: Handle,
    #[serde(deserialize_with = "deserialize_null_default")]
    name: String,
    stats: Stats,
    traits: Vec<String>,
}

impl Display for Champion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
