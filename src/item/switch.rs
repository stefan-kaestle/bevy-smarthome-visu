use crate::emoji;
use crate::events::LightModification;
use crate::events::SceneModificationEvent;
use crate::item::Item;
use crate::item::NotificationStatus;
use crate::openhab::RequestedStateChangeFromWidget;
use crate::openhab::WidgetInteraction;
use crate::ui;
use crate::widget_settings::EntityName;
use crate::widget_settings::SceneModification;
use crate::widget_settings::WidgetRenderSetting;

use bevy_egui::egui;
use bevy_egui::egui::Image;
use serde::Deserialize;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {};
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwitchConfig {
    icon: String,
    label: Option<String>,
}

pub struct Switch {
    state: bool,
    toggling: bool,
    watts: Option<f32>,
    icon: String,
    icon_given: bool,
    label: Option<String>,
}

impl Switch {
    pub fn new() -> Switch {
        Switch {
            state: false,
            toggling: false,
            watts: None,
            icon: "1F4A1".to_string(),
            icon_given: false,
            label: None,
        }
    }
}

impl Item for Switch {
    fn set_configuration(&mut self, config: &serde_json::Value) {
        let config: SwitchConfig =
            serde_json::from_value(config.clone()).expect("Failed to parse config for item");
        log!("Switch: using configuration {:?}", &config);
        self.icon = config.icon;
        self.icon_given = true;
        self.label = config.label;
    }

    fn state_changed(&mut self, key: &str, new_state: &str) {
        if key == "miliampere" {
            self.watts = Some(new_state.parse::<f32>().unwrap_or(0.) / 1000. * 230.);
        } else {
            self.state = new_state != "OFF";
            self.toggling = false;
        }
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
                        illuminance_percentage: if self.state { 1.0 } else { 0.0 },
                    },
                )]
            }
            //     "Hidden" => vec![BlenderConf::Hidden(BlenderConfHidden { val: !self.state })],
            //     "Show" => vec![BlenderConf::Hidden(BlenderConfHidden { val: self.state })],
            _ => {
                log!(
                    "Received unknown blender modification request: {}",
                    modification
                );
                vec![]
            }
        }
    }

    fn get_notification_status(&self) -> Option<NotificationStatus> {
        if self.state {
            Some(NotificationStatus {
                color: "yellow".to_string(),
                priority: 1,
                num: 1,
            })
        } else {
            None
        }
    }

    fn render_egui(
        &self,
        render_position: (f32, f32),
        render_setting: &WidgetRenderSetting,
        context: &mut egui::Context,
    ) -> Vec<WidgetInteraction> {
        let on_color = if self.icon_given {
            ui::DARK_GREEN
        } else {
            ui::DARK_YELLOW
        };

        ui::render_simple_button(
            context,
            render_setting,
            render_position,
            self.state,
            self.watts,
            emoji::get_emoji(&self.icon),
            on_color,
            None,
            None,
        )
    }
}
