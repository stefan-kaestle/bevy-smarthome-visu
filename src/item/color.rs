use crate::events::{LightModification, SceneModificationEvent};
use crate::openhab::{RequestedStateChangeFromWidget, WidgetInteraction};
use crate::{emoji, ui};
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32};

use crate::config;
use crate::item::{Item, NotificationStatus};
use crate::widget_settings::SceneModification;
use crate::widget_settings::{EntityName, WidgetRenderSetting};

macro_rules! log {
    ( $( $t:tt )* ) => {};
}

#[derive(Default)]
pub struct Color {
    state: [f32; 3],
    previous_lightness: f32,
    color: Option<Color32>,
}

// panicked at src/item/color.rs:23:18:
// index out of bounds: the len is 1 but the index is 1

impl Item for Color {
    fn state_changed(&mut self, _: &str, new_state: &str) {
        let split: Vec<&str> = new_state.split(',').collect();
        if split.len() == 3 {
            self.state = [
                split[0].parse::<f32>().unwrap_or(0.),
                split[1].parse::<f32>().unwrap_or(0.),
                split[2].parse::<f32>().unwrap_or(0.),
            ];

            let rgb = crate::openhab::openhab_hsb_to_rgb([self.state[0], self.state[1], 100.]);
            self.color = Some(Color32::from_rgb(
                (rgb[0] * 255.) as u8,
                (rgb[1] * 255.) as u8,
                (rgb[2] * 255.) as u8,
            ));
        } else {
            error!("Failed to parse state {} in Color", new_state);
            self.color = None;
        }
    }

    fn get_notification_status(&self) -> Option<NotificationStatus> {
        if self.state[2] != 0. {
            Some(NotificationStatus {
                color: "yellow".to_string(),
                priority: 1,
                num: 1,
            })
        } else {
            None
        }
    }

    fn state_to_blender(
        &self,
        entity_name: &EntityName,
        modification: SceneModification,
    ) -> Vec<SceneModificationEvent> {
        match modification {
            SceneModification::Energy(_) => {
                vec![SceneModificationEvent::LightModification(
                    LightModification {
                        entity_name: entity_name.to_string(),
                        illuminance_percentage: 1. / 100. * self.state[2],
                    },
                )]
            }
            _ => {
                error!(
                    "Unsupported modification {:?} in color widget",
                    modification
                );
                vec![]
            }
        }
    }

    /// Render a Switch
    fn render_egui(
        &self,
        render_position: (f32, f32),
        render_setting: &WidgetRenderSetting,
        context: &mut egui::Context,
    ) -> Vec<WidgetInteraction> {
        // XXX This should be cached.
        ui::render_simple_button(
            context,
            render_setting,
            render_position,
            self.state[2] != 0.,
            None,
            emoji::get_emoji(emoji::LIGHT_BULB),
            self.color.unwrap_or(ui::DARK_YELLOW),
            None,
            None,
        )
    }
}

impl Color {
    pub fn new() -> Color {
        Color {
            state: [0.0, 0.0, 0.0],
            previous_lightness: 100.,
            ..Default::default()
        }
    }
}
