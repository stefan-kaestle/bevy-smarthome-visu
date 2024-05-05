use crate::config::SmartHomeItemInternal;
use crate::events::SceneModificationEvent;
use crate::openhab::WidgetInteraction;
use crate::plot::PlotType;
use crate::widget_settings::{EntityName, SceneModification, WidgetRenderSetting};

use bevy::log::{debug, error};
use bevy_egui::egui;
use serde::Deserialize;
use std::collections::HashMap;

pub mod car;
pub mod climate;
pub mod color;
pub mod contact;
pub mod dimmer;
pub mod dimmer_group;
pub mod energy_monitor;
pub mod generic_item;
pub mod laundry;
pub mod light_auto;
pub mod robot;
pub mod switch;

pub struct NotificationStatus {
    pub color: String, // Any color string - will be passed as is to the template
    pub priority: u8,  // A priority as integer. 0 means nothing, higher means higher priority
    pub num: u32,      // Number of notifications for this configuration
}

pub(crate) trait Item {
    fn state_changed(&mut self, state_key: &str, new_state: &str);

    fn initial_state(&mut self, state_key: &str, new_state: &str) {
        self.state_changed(state_key, new_state);
    }

    fn set_configuration(&mut self, config: &serde_json::Value) {
        debug!("Ignoring configuration {:?}", config);
    }

    fn set_smarthome_items(&mut self, _config: &HashMap<String, SmartHomeItemInternal>) {}

    fn get_notification_status(&self) -> Option<NotificationStatus> {
        None
    }

    fn render_slider(&self) -> Option<(String, usize)> {
        None
    }

    fn get_plot_type(&self, _key: &str) -> PlotType {
        return PlotType::LinePlot;
    }

    fn state_to_blender(
        &self,
        _blender_item: &EntityName,
        _modfication: SceneModification,
    ) -> Vec<SceneModificationEvent> {
        vec![]
    }

    /// Render this widget using egui on the given context
    ///
    /// Widgets may return an HTTP Request response representing an event that should be triggered
    /// due to user interactions on the widget.
    ///
    /// By default, we don't render anything.
    fn render_egui(
        &self,
        _widget_position: (f32, f32),
        _widget_render_setting: &WidgetRenderSetting,
        _context: &mut egui::Context,
    ) -> Vec<WidgetInteraction> {
        vec![]
    }

    /// Render the fullscreen view of this widget, if enabled.
    fn render_fullscreen(
        &self,
        _widget_render_setting: &WidgetRenderSetting,
        ctx: &mut egui::Context,
    ) -> Vec<WidgetInteraction> {
        let mut requests = vec![];
        egui::Window::new("Fullscreen").show(ctx, |ui| {
            if ui.add(egui::Button::new("Close")).clicked() {
                requests.push(WidgetInteraction::FullscreenRequest(false));
            }
        });

        requests
    }
}

pub struct Number {
    value: f64,
}

impl Number {
    pub fn new() -> Number {
        Number { value: 0. }
    }
}

impl Item for Number {
    fn state_changed(&mut self, _: &str, new_state: &str) {
        let split: Vec<&str> = new_state.split(" ").collect();
        if let Ok(value) = split.get(0).unwrap().parse::<f64>() {
            self.value = value
        } else {
            debug!("Failed to parse number, got update: {}", new_state);
        }
    }
}

pub struct Blind {
    value: f64,
    running: bool,
    has_step: bool,
}

impl Blind {
    pub fn new() -> Blind {
        Blind {
            value: 0.,
            running: false,
            has_step: false,
        }
    }
}

impl Item for Blind {
    fn state_changed(&mut self, key: &str, new_state: &str) {
        if key != "step" {
            debug!("Blind::state_changed: {}", new_state);
            if let Ok(value) = new_state.parse::<f64>() {
                self.value = value;
                self.running = false;
            } else {
                debug!("Failed to parse number, got update: {}", new_state);
            }
        } else {
            self.has_step = true;
        }
    }
    fn get_notification_status(&self) -> Option<NotificationStatus> {
        if self.value > 0. {
            Some(NotificationStatus {
                color: "green".to_string(),
                priority: 1,
                num: 1,
            })
        } else {
            None
        }
    }

    fn get_plot_type(&self, _key: &str) -> PlotType {
        PlotType::StepPlot
    }
}

pub struct Text {
    value: Option<String>,
}

impl Text {
    pub fn new() -> Text {
        Text { value: None }
    }
}

impl Item for Text {
    fn state_changed(&mut self, _: &str, new_state: &str) {
        self.value = Some(new_state.to_string())
    }
}

pub struct Calendar {
    event: Option<String>,
    time: Option<String>,
}

impl Calendar {
    pub fn new() -> Calendar {
        Calendar {
            event: None,
            time: None,
        }
    }
}

impl Item for Calendar {
    fn state_changed(&mut self, state_key: &str, new_state: &str) {
        match state_key {
            "event" => self.event = Some(new_state.to_string()),
            "time" => self.time = Some(new_state.to_string()),
            _ => {
                error!(
                    "calendar: could not parse state_changed for key {} and state {}",
                    state_key, new_state
                );
            }
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MusicConfig {
    devices: Vec<[String; 2]>,
    id: String,
    radios: Vec<[String; 2]>,
    playlists: Vec<[String; 2]>,
}

pub struct Music {
    config: Option<MusicConfig>,
    controller: Option<String>,
    state: Option<String>,
    artist: Option<String>,
    title: Option<String>,
    track: Option<String>,
    image: Option<String>,
    zonename: Option<String>,
    zonegroupid: Option<String>,
    coordinator: Option<String>,
    volume: Option<u32>,
    // ts_artist: Instant,
    // ts_title: Instant,
    // ts_image: Instant,
    // ts_track: Instant,
}

impl Music {
    fn parse(&self, state: &str) -> Option<String> {
        if state.len() > 0 {
            Some(state.to_string())
        } else {
            None
        }
    }

    pub fn new() -> Music {
        Music {
            config: None,
            controller: None,
            artist: None,
            title: None,
            state: None,
            image: None,
            zonename: None,
            zonegroupid: None,
            coordinator: None,
            track: None,
            volume: None,
            // ts_artist: Instant::now(),
            // ts_track: Instant::now(),
            // ts_title: Instant::now(),
            // ts_image: Instant::now(),
        }
    }

    // pub fn unset_old(&mut self, now: Instant) {
    //     const MAX_TIME: Duration = Duration::from_secs(15);

    //     if now.duration_since(self.ts_artist) > MAX_TIME {
    //         self.artist = None;
    //     }
    //     if now.duration_since(self.ts_title) > MAX_TIME {
    //         self.title = None;
    //     }
    //     if now.duration_since(self.ts_image) > MAX_TIME {
    //         self.image = None;
    //     }
    // }

    fn is_standalone(&self) -> bool {
        self.config.as_ref().map(|x| x.id.to_string()) == self.coordinator
    }

    fn is_stopped(&self) -> bool {
        match &self.state {
            None => true,
            Some(state) => state == "STOPPED" || state == "PAUSED_PLAYBACK",
        }
    }
}

impl Item for Music {
    fn set_configuration(&mut self, config: &serde_json::Value) {
        let config: MusicConfig =
            serde_json::from_value(config.clone()).expect("Failed to parse config for item");
        debug!("Music: using configuration {:?}", &config);
        self.config = Some(config);
    }

    fn state_changed(&mut self, state_key: &str, new_state: &str) {
        // let now = Instant::now();

        let _unset = match state_key {
            "artist" => {
                self.artist = self.parse(new_state);
                // self.ts_artist = now;
                true
            }
            "title" => {
                self.title = self.parse(new_state);
                // self.ts_title = now;
                true
            }
            "track" => {
                self.track = self.parse(new_state);
                // self.ts_track = now;
                true
            }
            "image" => {
                self.image = self.parse(new_state);
                // self.ts_image = now;
                true
            }
            "state" => {
                self.state = self.parse(new_state);
                false
            }
            "zonename" => {
                self.zonename = self.parse(new_state);
                false
            }
            "zonegroupid" => {
                self.zonegroupid = self.parse(new_state);
                false
            }
            "coordinator" => {
                self.coordinator = self.parse(new_state);
                false
            }
            "volume" => {
                self.volume = new_state.parse::<u32>().ok();
                false
            }
            "controller" => {
                self.controller = self.parse(new_state);
                false
            }
            _ => {
                debug!(
                    "music: received update on unknown key {} to {}",
                    state_key, new_state
                );
                false
            }
        };

        // if unset {
        //     self.unset_old(now);
        // }
    }

    fn render_slider(&self) -> Option<(String, usize)> {
        if self.is_standalone() && !self.is_stopped() && self.image.is_some() {
            Some(("slider-music".to_string(), 100))
        } else {
            None
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SceneConfig {
    scenes: Vec<[String; 3]>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    scenes: Option<SceneConfig>,
}

impl Scene {
    pub fn new() -> Self {
        Self { scenes: None }
    }
}

impl Item for Scene {
    fn set_configuration(&mut self, config: &serde_json::Value) {
        self.scenes =
            Some(serde_json::from_value(config.clone()).expect("Failed to parse config for Scene"));
        debug!("Scene: using configuration {:?}", self.scenes);
    }

    fn state_changed(&mut self, _: &str, _new_state: &str) {}
}
