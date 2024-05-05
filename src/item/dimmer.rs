use bevy_egui::egui;

use crate::events::{LightModification, SceneModificationEvent};
use crate::openhab::{RequestedStateChangeFromWidget, WidgetInteraction};
use crate::{emoji, ui};

use crate::config::SmartHomeItemInternal;
use crate::item::{Item, NotificationStatus};
use crate::widget_settings::{EntityName, SceneModification, WidgetRenderSetting};

macro_rules! log {
    ( $( $t:tt )* ) => {};
}

use std::collections::HashMap;

const KEY_COLORTEMP: &str = "colortemp";

pub struct Dimmer {
    state: bool,
    value: u8,
    toggling: bool,
    color_temperature: Option<f32>,
    has_color_temperature: bool,
}

impl Dimmer {
    pub fn new(config: &HashMap<String, SmartHomeItemInternal>) -> Dimmer {
        let mut has_color_temperature = false;
        for (_, val) in config {
            if val.key == KEY_COLORTEMP {
                has_color_temperature = true;
            }
        }

        Dimmer {
            state: false,
            value: 0,
            toggling: false,
            color_temperature: None,
            has_color_temperature,
        }
    }
}

impl Item for Dimmer {
    fn state_changed(&mut self, key: &str, new_state: &str) {
        log!(
            "Recieved state_change in dimmer: new state is {}",
            new_state
        );

        if key == KEY_COLORTEMP {
            self.color_temperature = new_state.parse::<f32>().ok();
        } else {
            match new_state.parse::<u8>() {
                Ok(t) => self.value = t,
                Err(_) => {
                    // Not sure if that can actually happen anywhere else than in Simulation mode ..
                    if new_state == "ON" {
                        self.value = 100;
                    } else {
                        self.value = 0;
                    }
                }
            };

            self.state = self.value != 0;
            self.toggling = false;
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
                        illuminance_percentage: if self.state {
                            // self.value is in range 0..100
                            let mut f = self.value as f32;
                            if f < 10. {
                                f = 10.
                            }
                            // Convert to range 0..1
                            1. / 100. * f
                        } else {
                            0.0
                        },
                    },
                )]
                // vec![BlenderConf::Energy(BlenderConfEnergy {
                //     val: if self.state {
                //         let mut f = self.value as f32;
                //         if f < 10. {
                //             f = 10.
                //         }
                //         1. / 100. * f
                //     } else {
                //         0.0
                //     },
                // })]
            }
            //     "Hidden" => vec![BlenderConf::Hidden(BlenderConfHidden { val: !self.state })],
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
        if self.value > 0 {
            Some(NotificationStatus {
                color: "yellow".to_string(),
                priority: 1,
                num: 1,
            })
        } else {
            None
        }
    }

    /// Render a Switch
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
            self.state,
            None,
            emoji::get_emoji(emoji::LIGHT_BULB),
            ui::DARK_YELLOW,
            Some("100"),
            Some("0"),
        )
    }
}
