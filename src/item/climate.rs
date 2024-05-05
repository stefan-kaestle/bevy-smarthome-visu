use crate::config::SmartHomeItemInternal;
use crate::events::{SceneModificationEvent, SunModification};
use crate::widget_settings::{EntityName, SceneModification};

use instant::Instant;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use bevy::log::{error, info};

use crate::item::Item;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct ClimateConfig {
    sun_mapping: Vec<(f32, f32)>,
    world_mapping: Vec<(f32, f32)>,
    environment: HashMap<String, Vec<(f32, f32)>>,
}
pub struct Climate {
    humidity: Option<f64>,
    temperature: Option<f64>,
    set_temperature: Option<f64>,
    co2: Option<f64>,
    pm10: Option<f64>,
    purity: Option<f64>,
    illuminance: Option<f64>,
    azimuth: Option<f32>,
    wind: Option<f64>,
    elevation: Option<f32>,
    corona: Option<f64>,
    icp: Option<f64>,
    energy: Option<f64>,
    power: Option<f64>,
    last_update: Instant,
    items: HashMap<String, String>,
    config: Option<ClimateConfig>,
}

impl Climate {
    fn parse(&self, state: &str) -> Option<f64> {
        // Could also just remove all non-number characters, but that sounds less stable.
        let state = state.replace("°C", "");
        let state = state.replace("°", "");
        let state = state.replace("lx", "");
        let state = state.replace("km/h", "");
        let state = state.replace("W", "");
        let state = state.trim();

        match state.parse::<f64>() {
            Ok(v) => Some(v),
            Err(e) => {
                if state != "NULL" {
                    error!(
                        "climate: failed to parse value {} to float - {:?}",
                        state, e
                    );
                }

                None
            }
        }
    }

    pub fn new() -> Climate {
        Climate {
            humidity: None,
            temperature: None,
            set_temperature: None,
            co2: None,
            pm10: None,
            purity: None,
            illuminance: None,
            corona: None,
            icp: None,
            energy: None,
            power: None,
            azimuth: None,
            wind: None,
            elevation: None,
            last_update: Instant::now(),
            items: HashMap::new(),
            config: None,
        }
    }

    pub fn do_render(&mut self) -> bool {
        if self.last_update.elapsed() > Duration::from_secs(60 * 5) {
            self.last_update = Instant::now();
            true
        } else {
            false
        }
    }
}

impl Item for Climate {
    fn set_configuration(&mut self, config: &serde_json::Value) {
        let config: ClimateConfig =
            serde_json::from_value(config.clone()).expect("Failed to parse config for item");
        info!("Climate: using configuration {:?}", &config);
        self.config = Some(config);
    }

    fn set_smarthome_items(&mut self, config: &HashMap<String, SmartHomeItemInternal>) {
        for (item, value) in config {
            info!("Climate items: {} -> {}", value.key, item);
            self.items.insert(value.key.clone(), item.to_string());
        }
    }

    fn state_changed(&mut self, state_key: &str, new_state: &str) {
        match state_key {
            "humidity" => self.humidity = self.parse(new_state),
            "purity" => self.purity = self.parse(new_state),
            "co2" => self.co2 = self.parse(new_state),
            "pm10" => self.pm10 = self.parse(new_state),
            "temperature" => self.temperature = self.parse(new_state),
            "set-temperature" => self.set_temperature = self.parse(new_state),
            "illuminance" => self.illuminance = self.parse(new_state),
            "corona" => self.corona = self.parse(new_state),
            "icp" => self.icp = self.parse(new_state),
            "power" => self.power = self.parse(new_state),
            "energy" => self.energy = self.parse(new_state),
            "azimuth" => self.azimuth = self.parse(new_state).map(|x| x as f32),
            "wind" => self.wind = self.parse(new_state),
            "elevation" => self.elevation = self.parse(new_state).map(|x| x as f32),
            _ => {
                error!(
                    "climate: received update on unknown key {} to {}",
                    state_key, new_state
                );
            }
        }
    }

    fn state_to_blender(
        &self,
        _blender_item: &EntityName,
        modification: SceneModification,
    ) -> Vec<SceneModificationEvent> {
        if let Some(illuminance) = self.illuminance {
            //     let default_world_mapping = vec![(0., 0.03), (10000., 0.33)];

            //     let world_mapping = match &self.config {
            //         Some(config) => config.world_mapping.clone(),
            //         None => default_world_mapping,
            //     };

            // Value from which we set the maximum energy.

            let azimuth = self.azimuth.unwrap_or(180.);
            let elevation = self.elevation.unwrap_or(40.);

            //     let world_color = scale_value(illuminance as f32, &world_mapping);
            //     log!(
            //         "Sun: setting current energy to {} - current illumination {} - world color {} - rotation {},{}",
            //         sun_energy,
            //         illuminance,
            //         world_color,
            //         azimuth,
            //         elevation,
            //     );

            match modification {
                SceneModification::Sun() => {
                    // let default_sun_mapping = vec![(0., 0.), (30000., 15.)];
                    // let sun_mapping = match &self.config {
                    //     Some(config) => config.sun_mapping.clone(),
                    //     None => default_sun_mapping,
                    // };

                    // let sun_energy = scale_value(illuminance as f32, &sun_mapping);

                    vec![
                        SceneModificationEvent::SunModification(SunModification {
                            illuminance,
                            elevation,
                            azimuth,
                        }),
                        // BlenderConf::World(BlenderConfWorld {
                        //     val: [world_color; 3],
                        // }),
                        // BlenderConf::RotationX(BlenderConfRotation { val: elevation }),
                        // BlenderConf::RotationY(BlenderConfRotation { val: 0. }),
                        // BlenderConf::RotationZ(BlenderConfRotation { val: azimuth }),
                    ]
                }
                _ => {
                    error!("Currently unsupported modification {:?}", modification);
                    vec![]
                }
            }
        //             if let Some(config) = &self.config {
        //                 let mut v = vec![];
        //                 for (light, mapping) in &config.environment {
        //                     debug!("Checking requested blender light {} against configuration item given as {}", blender_item, light);
        //                     if light == blender_item {
        //                         let energy_val = scale_value(illuminance as f32, &mapping);
        //                         v.push(BlenderConf::Energy(BlenderConfEnergy { val: energy_val }));
        //                     }
        //                 }
        //                 v
        //             } else {
        //                 log!(
        //                     "Received unknown blender modification request: {}",
        //                     modification
        //                 );
        //                 vec![]
        //             }
        //         }
        //     }
        } else {
            vec![]
        }
    }
}
