use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::item::generic_item::{generic_translate_value, GenericItem};

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LaundryConfig {
    pub image: String,
}

fn init_map(generic_item: &GenericItem<LaundryConfig>) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    if let Some(config) = &generic_item.config {
        map.insert("image".to_string(), Value::String(config.image.to_string()));
    }
    map
}

fn render_slider(generic_item: &GenericItem<LaundryConfig>) -> Option<(String, usize)> {
    match generic_item.state.get("active") {
        Some(state) => {
            if state.0 == "ON" {
                Some(("slider-laundry".to_string(), 15))
            } else {
                None
            }
        }
        None => None,
    }
}

pub fn new() -> GenericItem<LaundryConfig> {
    GenericItem::with_custom_functions(generic_translate_value, init_map, render_slider)
}
