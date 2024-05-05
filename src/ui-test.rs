#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

// pub mod blender;
pub mod config;
// pub mod config_loader;
// //pub mod device_model;
pub mod emoji;
pub mod errors;
// pub mod events;
// //pub mod item;
pub mod openhab;
// pub mod plot;
pub mod ui;
pub mod widget_settings;

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32},
    EguiContexts, EguiPlugin,
};
use emoji::get_emoji;
use widget_settings::WidgetRenderSetting;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
        .add_systems(Update, ui_example_system)
        .run();
}

fn ui_example_system(mut contexts: EguiContexts) {
    let ctx: &egui::Context = contexts.ctx_mut();
    egui_extras::install_image_loaders(ctx);

    let ctx = contexts.ctx_mut();
    ui::render_simple_button(
        ctx,
        &WidgetRenderSetting {
            top: 50.,
            left: 50.,
            id: "test".into(),
            label: Some("Label".to_string()),
            widget_name: "WidgetName".to_string(),
        },
        (50., 50.),
        true,
        Some(1234.),
        get_emoji("E150"),
        ui::DARK_GREEN,
        None,
        None,
    );
    ui::render_simple_button(
        ctx,
        &WidgetRenderSetting {
            top: 50.,
            left: 200.,
            id: "test2".into(),
            label: None,
            widget_name: "WidgetName2".to_string(),
        },
        (50., 200.),
        true,
        None,
        get_emoji("1F4A1"),
        ui::DARK_YELLOW,
        None,
        None,
    );
    ui::render_simple_button(
        ctx,
        &WidgetRenderSetting {
            top: 100.,
            left: 50.,
            id: "test3".into(),
            label: None,
            widget_name: "WidgetName3".to_string(),
        },
        (100., 50.),
        false,
        None,
        get_emoji("1F4A1"),
        ui::DARK_YELLOW,
        None,
        None,
    );
}
