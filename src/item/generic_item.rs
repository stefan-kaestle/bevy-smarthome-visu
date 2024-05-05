use crate::config::SmartHomeItemInternal;
use crate::events::SceneModificationEvent;
use crate::item::Item;
use crate::openhab::{RequestedStateChangeFromWidget, WidgetInteraction};
use crate::widget_settings::EntityName;
use crate::widget_settings::{SceneModification, WidgetRenderSetting};

use bevy_egui::egui;
use chrono_humanize::{Accuracy, HumanTime, Tense};
use instant::Instant;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

use super::NotificationStatus;

macro_rules! error {
    ( $( $t:tt )* ) => {};
}
macro_rules! log {
    ( $( $t:tt )* ) => {};
}

pub const INTERNAL_STATE_PREFIX: &str = "@";

pub struct GenericItem<T: DeserializeOwned> {
    // Stores the current value, indexed by key.
    pub(crate) state: HashMap<String, (String, Option<Instant>)>,
    // Map OpenHab name to key
    pub(crate) items: HashMap<String, String>,
    pub(crate) config: Option<T>,
    translate_f: fn(&str, &str) -> Value,
    init_f: fn(&GenericItem<T>) -> serde_json::Map<String, Value>,
    render_slider_f: fn(&GenericItem<T>) -> Option<(String, usize)>,
    blender_f: Option<fn(&GenericItem<T>, &str, &str, f32) -> Vec<SceneModificationEvent>>,
    notification_f: Option<fn(&GenericItem<T>) -> Option<NotificationStatus>>,
}

impl<T: DeserializeOwned> Default for GenericItem<T> {
    fn default() -> Self {
        GenericItem {
            state: HashMap::default(),
            config: None,
            items: HashMap::default(),
            translate_f: generic_translate_value,
            init_f: generic_init_map,
            render_slider_f: generic_render_slider,
            blender_f: None,
            notification_f: None,
            ..Default::default()
        }
    }
}

impl<T: DeserializeOwned> GenericItem<T> {
    pub(crate) fn with_custom_functions(
        f: fn(&str, &str) -> Value,
        init_f: fn(&GenericItem<T>) -> serde_json::Map<String, Value>,
        render_slider_f: fn(&GenericItem<T>) -> Option<(String, usize)>,
    ) -> Self {
        GenericItem {
            translate_f: f,
            init_f,
            render_slider_f,
            ..Default::default()
        }
    }

    pub fn with_notification_f<'a>(
        &'a mut self,
        f: fn(&GenericItem<T>) -> Option<NotificationStatus>,
    ) -> &'a mut Self {
        self.notification_f = Some(f);
        self
    }

    pub fn with_blender_f<'a>(
        &'a mut self,
        f: fn(&GenericItem<T>, &str, &str, f32) -> Vec<SceneModificationEvent>,
    ) -> &'a mut Self {
        self.blender_f = Some(f);
        self
    }

    pub fn with_init_f<'a>(
        &'a mut self,
        f: fn(&GenericItem<T>) -> serde_json::Map<String, Value>,
    ) -> &'a mut Self {
        self.init_f = f;
        self
    }
}

impl<T: DeserializeOwned> Item for GenericItem<T> {
    fn set_configuration(&mut self, config: &serde_json::Value) {
        self.config = Some(
            serde_json::from_value(config.clone())
                .expect("Failed to parse config for generic item."),
        );
    }

    fn set_smarthome_items(&mut self, items: &HashMap<String, SmartHomeItemInternal>) {
        for (item_name, value) in items {
            self.items
                .insert(item_name.to_string(), value.key.to_string());
        }
    }

    fn state_changed(&mut self, item_key: &str, new_state: &str) {
        let timestamp = match self.state.get(item_key) {
            Some(old_val) => {
                if old_val.0 == new_state {
                    old_val.1.clone()
                } else {
                    Some(Instant::now())
                }
            }
            None => Some(Instant::now()),
        };

        self.state
            .insert(item_key.to_string(), (new_state.to_string(), timestamp));
    }

    fn initial_state(&mut self, item_key: &str, new_state: &str) {
        self.state
            .insert(item_key.to_string(), (new_state.to_string(), None));
    }

    fn render_slider(&self) -> Option<(String, usize)> {
        (self.render_slider_f)(&self)
    }

    fn render_egui(
        &self,
        render_position: (f32, f32),
        render_setting: &WidgetRenderSetting,
        context: &mut egui::Context,
    ) -> Vec<WidgetInteraction> {
        vec![]
    }

    fn get_notification_status(&self) -> Option<super::NotificationStatus> {
        None
    }
}

pub(crate) fn generic_translate_value(_key: &str, input: &str) -> Value {
    match input {
        "ON" => Value::Bool(true),
        "OFF" => Value::Bool(false),
        _ => Value::String(input.to_string()),
    }
}

pub(crate) fn generic_init_map<T: DeserializeOwned>(
    _: &GenericItem<T>,
) -> serde_json::Map<String, Value> {
    serde_json::Map::new()
}

pub(crate) fn generic_render_slider<T: DeserializeOwned>(
    _: &GenericItem<T>,
) -> Option<(String, usize)> {
    None
}

pub(crate) fn generic_render_egui<T: DeserializeOwned>(
    _item: &T,
    _render_position: (f32, f32),
    _render_setting: &WidgetRenderSetting,
    _context: &mut egui::Context,
) -> Vec<RequestedStateChangeFromWidget> {
    vec![]
}
