extern crate colortemp;
// extern crate wasm_bindgen_test;

use bevy::log::error;
use bevy_egui::egui::{self, Color32};

use crate::events::{LightModification, SceneModificationEvent};
use crate::openhab::{RequestedStateChangeFromWidget, WidgetInteraction};

use crate::widget_settings::{EntityName, SceneModification, WidgetRenderSetting};
use crate::{emoji, ui};

use super::Item;

#[derive(Clone, Copy, Debug)]
enum AggregateType {
    DimmAll(),
    DimmUp(),
    DimmDown(),
    ColorTemp(),
}

#[derive(Default)]
pub(crate) struct DimmerGroup {
    // Keys: updated when the state updates
    dimmer1: f32,
    dimmer1up: f32,
    dimmer2: f32,
    dimmer2up: f32,
    colortemp1: f32,
    colortemp2: f32,
    all: f32,
    allup: f32,
    alldown: f32,
    allcolortemp: f32,
    // Aggregates: we calculate these after each state update
    avg_dimm_all: f32,
    avg_dimm_up: f32,
    avg_dimm_down: f32,
    avg_color_temp: f32,
    // Cahed for rendering
    color: Color32,
}

impl DimmerGroup {
    fn get_average_value(&self, aggregate_type: AggregateType) -> f32 {
        let dimm_values = match &aggregate_type {
            AggregateType::DimmAll() => {
                vec![self.dimmer1, self.dimmer1up, self.dimmer2, self.dimmer2up]
            }
            AggregateType::DimmUp() => vec![self.dimmer1up, self.dimmer2up],
            AggregateType::DimmDown() => vec![self.dimmer1, self.dimmer2],
            AggregateType::ColorTemp() => vec![self.colortemp1, self.colortemp2],
        };
        let dimm_values = dimm_values.iter();
        let r = dimm_values.clone().sum::<f32>() / (dimm_values.count() as f32);
        if r.is_nan() {
            0.0
        } else {
            r
        }
    }

    fn update_aggregates(&mut self) {
        self.avg_dimm_all = self.get_average_value(AggregateType::DimmAll());
        self.avg_dimm_up = self.get_average_value(AggregateType::DimmUp());
        self.avg_dimm_down = self.get_average_value(AggregateType::DimmDown());
        self.avg_color_temp = self.get_average_value(AggregateType::ColorTemp());
    }

    fn is_on(&self) -> bool {
        self.avg_dimm_all.ceil() > 0.
    }

    fn update_color(&mut self) {
        let rgb = colortemp::temp_to_rgb(self.avg_color_temp as i64);
        self.color = Color32::from_rgb(rgb.r as u8, rgb.g as u8, rgb.b as u8)
    }
}

impl Item for DimmerGroup {
    fn state_changed(&mut self, state_key: &str, new_state: &str) {
        match new_state.parse::<f32>() {
            Ok(new_state) => match state_key {
                "dimmer1" => self.dimmer1 = new_state / 100.,
                "dimmer1up" => self.dimmer1up = new_state / 100.,
                "dimmer2" => self.dimmer2 = new_state / 100.,
                "dimmer2up" => self.dimmer2up = new_state / 100.,
                "colortemp1" => self.colortemp1 = new_state,
                "colortemp2" => self.colortemp2 = new_state,
                "all" => self.all = new_state,
                "allup" => self.allup = new_state,
                "alldown" => self.alldown = new_state,
                "allcolortemp" => self.allcolortemp = new_state,
                _ => error!("Received unknown key {} in DimmerGroup", state_key),
            },
            Err(e) => error!("Failed to parse state {} in DimmerGroup", e),
        }
        self.update_aggregates();
        self.update_color();
    }

    fn state_to_blender(
        &self,
        entity_name: &EntityName,
        modification: SceneModification,
    ) -> Vec<SceneModificationEvent> {
        match modification {
            SceneModification::Energy(_illuminance) => {
                vec![SceneModificationEvent::LightModification(
                    LightModification {
                        entity_name: entity_name.to_string(),
                        illuminance_percentage: self.avg_dimm_all,
                    },
                )]
            }
            _ => {
                error!("Unsupported mofication {:?} in DimmerGroup", modification);
                vec![]
            }
        }
    }

    fn render_egui(
        &self,
        render_position: (f32, f32),
        render_setting: &WidgetRenderSetting,
        context: &mut egui::Context,
    ) -> Vec<WidgetInteraction> {
        ui::render_simple_button(
            context,
            render_setting,
            render_position,
            self.is_on(),
            None,
            emoji::get_emoji("1F4A1"),
            self.color,
            Some("100"),
            Some("0"),
        )
    }
}
