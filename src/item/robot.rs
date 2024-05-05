use serde::Deserialize;
use serde_json::Value;
use std::collections::hash_map::Iter;
use std::collections::HashMap;

use crate::item::generic_item::{generic_translate_value, GenericItem, INTERNAL_STATE_PREFIX};

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RobotConfig {
    image: String,
    pmap_id: String,
    user_pmap_id: String,
    rooms: HashMap<String, String>,
    regions: HashMap<String, String>,
}

fn init_map(generic_item: &GenericItem<RobotConfig>) -> serde_json::Map<String, Value> {
    let is_home = match generic_item.state.get("phase") {
        Some(state) => state.0 == "charge",
        None => false,
    };
    let mut map = serde_json::Map::new();
    if let Some(config) = &generic_item.config {
        map.insert("image".to_string(), Value::String(config.image.to_string()));
        map.insert("is_home".to_string(), Value::Bool(is_home));

        let default = "false".to_string();

        let translate = |iter: Iter<String, String>| {
            let mut res: Vec<(String, String, bool)> = vec![];
            let mut selection: Vec<(String, String)> = vec![];

            for (name, id) in iter {
                let select_value = generic_item
                    .state
                    .get(&format!("{}select_{}", INTERNAL_STATE_PREFIX, &name))
                    .map(|s| &s.0)
                    .unwrap_or(&default)
                    == &"true";
                if select_value {
                    selection.push((name.to_string(), id.to_string()));
                }
                res.push((name.to_string(), id.to_string(), select_value));
            }

            return (res, selection);
        };

        let (rooms, mut selection) = translate(config.rooms.iter());
        map.insert(
            "rooms".to_string(),
            serde_json::to_value(rooms).unwrap_or(Value::Null),
        );

        let (regions, mut region_selection) = translate(config.regions.iter());
        selection.append(&mut region_selection);

        map.insert(
            "regions".to_string(),
            serde_json::to_value(regions).unwrap_or(Value::Null),
        );

        map.insert(
            "has_selection".to_string(),
            serde_json::to_value(selection.len() > 0).unwrap(),
        );

        let command_string = selection
            .into_iter()
            .map(|(_, id)| id)
            .collect::<Vec<String>>()
            .join(",");

        let clean_command = format!(
            "cleanRegions:{};{};{}",
            config.pmap_id, command_string, config.user_pmap_id
        );
        map.insert(
            "clean_command".to_string(),
            serde_json::to_value(clean_command).unwrap(),
        );
    }
    map
}

fn render_slider(generic_item: &GenericItem<RobotConfig>) -> Option<(String, usize)> {
    match generic_item.state.get("phase") {
        Some(state) => {
            if state.0 != "charge" {
                Some(("slider-robot".to_string(), 20))
            } else {
                None
            }
        }
        None => None,
    }
}

pub(crate) fn new() -> GenericItem<RobotConfig> {
    GenericItem::with_custom_functions(generic_translate_value, init_map, render_slider)
}
