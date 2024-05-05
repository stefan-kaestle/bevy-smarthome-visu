use serde::{Deserialize, Serialize};
use serde_json::Value;

use chrono::offset::Utc;
use chrono::{DateTime, ParseError};
use chrono_humanize::{Accuracy, HumanTime, Tense};

use crate::item::generic_item::{generic_translate_value, GenericItem};

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CarConfig {
    pub image: String,
}

fn translate_value(key: &str, input: &str) -> Value {
    match key {
        "odometer" => Value::String(
            input
                .replace(" m", "")
                .parse::<f32>()
                .map(|s| format!("{:.0} km", s / 1000.))
                .map_err(|e| e.to_string())
                .unwrap_or(input.to_string()),
        ),
        "eventstamp" => Value::String(
            input
                .parse::<DateTime<Utc>>()
                .map(|d| HumanTime::from(Utc::now() - d).to_text_en(Accuracy::Rough, Tense::Past))
                .map_err(|e: ParseError| e.to_string())
                .unwrap_or(input.to_string()),
        ),
        _ => generic_translate_value(key, input),
    }
}

fn init_map(generic_item: &GenericItem<CarConfig>) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    if let Some(config) = &generic_item.config {
        map.insert("image".to_string(), Value::String(config.image.to_string()));
    }
    map
}

fn render_slider(_generic_item: &GenericItem<CarConfig>) -> Option<(String, usize)> {
    Some(("slider-car".to_string(), 10))
}

pub(crate) fn new() -> GenericItem<CarConfig> {
    GenericItem::with_custom_functions(translate_value, init_map, render_slider)
}
