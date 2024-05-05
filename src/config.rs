use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ItemConfiguration {
    pub views: HashMap<String, ViewConfiguration>,
    pub zooms: HashMap<String, HashMap<String, (f64, f64, f64, f64)>>,
}

impl ItemConfiguration {
    /// Return the widget name and key for the given item, if it exists.
    ///
    /// If multiple widgets names for the same item exist, the first one will be returned.
    pub fn get_widget_for_item(&self, item_name: &str) -> Option<(String, String)> {
        for (_, view_configuration) in &self.views {
            for (widget_name, item) in &view_configuration.items {
                for (curr_item_name, item) in &item.smarthome_items {
                    if curr_item_name == item_name {
                        let key = item.key.as_ref().unwrap_or(curr_item_name).clone();
                        return Some((widget_name.to_string(), key));
                    }
                }
            }
        }
        None
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ViewConfiguration {
    pub image: Option<String>,
    pub label: Option<String>,
    pub order: i32,
    pub default: Option<bool>,
    pub blender_camera: Option<String>,
    pub blender_hide: Vec<String>,
    pub static_image: Option<String>,
    pub items: HashMap<String, Item>,
}

#[derive(Clone, Debug)]

/// External representation of a smart home item.
/// The key is optional, and if not given, will be the same
/// as the name of the smart home item.
pub struct SmartHomeItemInternal {
    pub key: String,
}

/// Internal representation of a smart home item.
/// Needs to have a key set.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SmartHomeItem {
    pub key: Option<String>,
}

pub fn smart_home_item_to_internal(
    i: &HashMap<String, SmartHomeItem>,
) -> HashMap<String, SmartHomeItemInternal> {
    let mut map = HashMap::new();
    for (key, value) in i {
        map.insert(
            key.to_string(),
            SmartHomeItemInternal {
                key: value.key.as_ref().unwrap_or(key).to_string(),
            },
        );
    }
    map
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub blender_items: HashMap<String, Vec<String>>,
    pub item_type: String,
    pub template: Option<String>,
    pub template_conf: Option<serde_json::Value>,
    pub energy_max: Option<f32>,
    pub top: Option<String>,
    pub left: Option<String>,
    pub smarthome_items: HashMap<String, SmartHomeItem>,
    pub label: Option<String>,
    pub show_mobile: Option<bool>,
}

impl Item {
    pub fn get_template(&self) -> String {
        let template: String = match &self.template {
            Some(t) => t.to_string(),
            None => "basic-light".to_string(),
        };
        format!("{}", template)
    }
}

#[derive(Debug)]
pub struct View {
    pub image: Option<String>,
    pub label: Option<String>,
    pub blender_camera: Option<String>,
    pub static_image: Option<String>,
    pub blender_hide: Vec<String>,
    pub ui_elements: Vec<String>, // These are widget_names, key to item_list
    pub order: i32,
}
