use serde::Deserialize;
use serde_json::Value;

use super::generic_item::{generic_render_slider, generic_translate_value, GenericItem};

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LightAutoConfig {}

fn init_map(generic_item: &GenericItem<LightAutoConfig>) -> serde_json::Map<String, Value> {
    let auto_on = match generic_item.state.get("disable_auto") {
        Some(state) => state.0 == "OFF",
        None => false,
    };
    let mut map = serde_json::Map::new();

    map.insert("d_auto_on".to_string(), Value::Bool(auto_on));
    if let Some(lux_set_is) = generic_item.state.get("lux_set") {
        map.insert(
            format!("lux_set_is_equal_{}", lux_set_is.0),
            Value::Bool(true),
        );
    }
    map
}

pub(crate) fn new() -> GenericItem<LightAutoConfig> {
    GenericItem::with_custom_functions(generic_translate_value, init_map, generic_render_slider)
}
