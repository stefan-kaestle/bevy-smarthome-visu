use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::HashMap;

use crate::config::ItemConfiguration;
use crate::errors::DeviceModelError;
use crate::events::SceneModificationEvent;
use crate::openhab::{self, RequestedStateChange};
use crate::widget_settings::*;

use crate::config::smart_home_item_to_internal;
use crate::item;
use crate::item::Item;

#[derive(Default, Resource)]
pub struct DeviceModel {
    /// Set to true when the json config is parsed.
    pub(crate) initialized: bool,

    /// Views with their corresponding widgets for this model.
    pub(crate) views: HashMap<ViewName, Vec<WidgetName>>,

    /// Widgets with the corresponding Item representation.
    pub(crate) widgets: HashMap<WidgetName, Box<dyn Item + Send + Sync>>,

    /// Settings for rendering the widget to the screen.
    /// Each widget must have an item list. However, it might not have render settings
    /// if the widget is not rendered (e.g. if it only has a slider.)
    widget_settings: HashMap<WidgetName, WidgetSettings>,

    /// Currently active view.
    pub(crate) current_view: Option<ViewName>,

    /// For each smart home item, a list of widgets with their keys.
    /// This is used to lookup which widgets needs to be informed of
    /// status updates from the smart home system.
    pub(crate) smart_home_items: HashMap<ItemName, Vec<(WidgetName, WidgetKey)>>,

    /// Widget which is in fullscreen mode, if any.
    pub(crate) fullscreen_widget: Option<WidgetName>,
}

impl DeviceModel {
    /// Instantiate widget from widget type and configuration.
    fn init_widget(
        widget_type: &str,
        widget_config: &crate::config::Item,
    ) -> Box<dyn Item + Send + Sync> {
        match widget_type {
            "Color" => Box::new(item::color::Color::new()),
            "Switch" => Box::new(item::switch::Switch::new()),
            "Dimmer" => Box::new(item::dimmer::Dimmer::new(&smart_home_item_to_internal(
                &widget_config.smarthome_items,
            ))),
            "DimmerGroup" => Box::new(item::dimmer_group::DimmerGroup::default()),
            // "Number" => Box::new(item::Number::new()),
            // "Blind" => Box::new(item::Blind::new()),
            // "Contact" => Box::new(item::contact::new()),
            // "Text" => Box::new(item::Text::new()),
            // "Calendar" => Box::new(item::Calendar::new()),
            // "Climate" => Box::new(item::climate::Climate::new()),
            // "Music" => Box::new(item::Music::new()),
            // "Scene" => Box::new(item::Scene::new()),
            // "Car" => Box::new(item::car::new()),
            // "Robot" => Box::new(item::robot::new()),
            // "EnergyMonitor" => Box::new(item::energy_monitor::new()),
            // "Laundry" => Box::new(item::laundry::new()),
            // "LightAuto" => Box::new(item::light_auto::new()),
            _ => {
                error!(
                    "Error in configuration - unknown item type: {}",
                    widget_type
                );
                Box::new(item::Number::new())
            }
        }
    }

    /// Build data structure to map from OpenHab name to widget and corresponding key.
    fn parse_smarthome_items_for_widget(
        &mut self,
        widget_name: &str,
        widget_config: &crate::config::Item,
    ) {
        for (item_name, value) in &widget_config.smarthome_items {
            let key: WidgetKey = value.key.as_ref().unwrap_or(item_name).to_string();
            self.smart_home_items
                .entry(item_name.to_string())
                .or_insert(Vec::new())
                .push((widget_name.to_string(), key));
        }

        // This widget does not have any smart home items explictly configured. Assume item name
        // equals widget name.
        if widget_config.smarthome_items.len() == 0 {
            // Since we do not have any smart home items for this widget, it should not be in the
            // hashmap yet. We check the return code of insert() for this.
            let previous = self.smart_home_items.insert(
                widget_name.to_string(),
                vec![(widget_name.to_string(), widget_name.to_string())],
            );
            assert!(
                previous.is_none(),
                "Duplicate widget name {} - previous value: {:?}, config: {:?}",
                &widget_name,
                &previous,
                &widget_config
            );
        }
    }

    pub fn parse(&mut self, configuration: &ItemConfiguration) {
        // XXX - Choose view based on ordering in json ..
        assert!(
            self.current_view.is_none(),
            "Current view already set, redundant call to parse configuration?"
        );
        self.current_view = Some("floorplan".to_string());

        // Parse configuration
        for (view_name, view) in &configuration.views {
            let mut widgets_of_view = vec![];

            for (widget_name, widget_config) in &view.items {
                widgets_of_view.push(widget_name.to_string());
                let widget_type: &str = &widget_config.item_type;

                // Get widget instance based on the widget type
                let mut widget = DeviceModel::init_widget(widget_type, widget_config);

                // Pass configuration to widget
                if let Some(template_conf) = &widget_config.template_conf {
                    widget.set_configuration(template_conf);
                }

                // Pass list of smarthome items to widget
                widget.set_smarthome_items(&smart_home_item_to_internal(
                    &widget_config.smarthome_items,
                ));

                // Store reference to widget
                self.widgets.insert(widget_name.to_string(), widget);

                // Store settings for widgets. Those include information needed for rendering as well
                // as information for lookup from key to item.
                self.widget_settings.insert(
                    widget_name.to_string(),
                    WidgetSettings {
                        render_settings: WidgetRenderSetting::from_widget_config(
                            view_name,
                            widget_name,
                            widget_config,
                        ),
                        item_list: WidgetItemList::from_widget_config(widget_name, widget_config),
                        scene_modifications: WidgetSceneModifications::from_widget_config(
                            &widget_config.blender_items,
                        ),
                    },
                );

                self.parse_smarthome_items_for_widget(widget_name, widget_config);
            }

            // Remember all widgets to be displayed for that view
            self.views.insert(view_name.to_string(), widgets_of_view);
        }

        self.initialized = true;
    }

    /// Render all widgets for the current view using Egui.
    ///
    /// If the current view does not exist or does not contain any valid widgets,
    /// nothing is being rendered.
    ///
    /// Returns a list of HTTP requests events to be appended to the corresponding event writer.
    ///
    /// XXX Needs better error handling instead of using Options all over the place.
    pub(crate) fn render(
        &mut self,
        context: &mut egui::Context,
        camera: &Camera,
        camera_global_transform: &GlobalTransform,
    ) -> Result<Vec<RequestedStateChange>, DeviceModelError> {
        let mut requests = vec![];

        // Current view is set
        let current_view = self
            .current_view
            .as_ref()
            .ok_or(DeviceModelError::NoViewSelected(()))?;

        // Current view exists
        let widgets = self
            .views
            .get(current_view)
            .ok_or(DeviceModelError::ViewNotFound(current_view.to_string()))?;

        for widget_name in widgets {
            // Widget has been registered with a position
            let widget_settings = self.widget_settings.get(widget_name).ok_or(
                DeviceModelError::WidgetSettingsNotFound(widget_name.to_string()),
            )?;

            // Check if the item has render settings. If it does not have them,
            // we don't want to render it.
            if let Some(render_settings) = &widget_settings.render_settings {
                // Item impl for widget with that widget name is found
                let widget = self
                    .widgets
                    .get(widget_name)
                    .ok_or(DeviceModelError::WidgetNotFound(widget_name.to_string()))?;

                // Get viewport position for the given 3D coordinates via camera.
                // https://github.com/bevyengine/bevy/blob/release-0.13.2/examples/3d/blend_modes.rs
                let viewport_position = camera.world_to_viewport(
                    camera_global_transform,
                    Vec3::new(render_settings.left, 2.0, render_settings.top),
                );

                let mut widget_requests = vec![];

                // If visible with current camera,
                // render widget and append HTTP requests triggerd by those widgets
                if let Some(viewport_position) = viewport_position {
                    widget_requests.append(&mut widget.render_egui(
                        (viewport_position.x, viewport_position.y),
                        &render_settings,
                        context,
                    ));
                }

                // Render fullscreen view, if requested.
                if self.fullscreen_widget.as_ref() == Some(widget_name) {
                    info_once!("Rendering fullscreen: {}", widget_name);
                    widget_requests
                        .append(&mut widget.render_fullscreen(&render_settings, context));
                }

                for widget_request in widget_requests {
                    match widget_request {
                        openhab::WidgetInteraction::StateChange(state_change) => {
                            requests.push(RequestedStateChange::from_widget_request(
                                &state_change,
                                &widget_settings.item_list,
                            )?)
                        }
                        openhab::WidgetInteraction::FullscreenRequest(enable) => match enable {
                            true => self.fullscreen_widget = Some(widget_name.to_string()),
                            false => self.fullscreen_widget = None,
                        },
                    }
                }
            }
        }

        Ok(requests)
    }

    pub(crate) fn state_changed(
        &mut self,
        item_name: &str,
        state: &str,
    ) -> Result<Vec<SceneModificationEvent>, DeviceModelError> {
        bevy::log::info!("State update: {} <- {}", item_name, state);
        let mut scene_changes = vec![];

        let widgets = self
            .smart_home_items
            .get(item_name)
            .ok_or(DeviceModelError::ItemNotFound(item_name.to_string()))?;

        for (widget_name, key) in widgets {
            let widget = self
                .widgets
                .get_mut(widget_name)
                .ok_or(DeviceModelError::WidgetNotFound(widget_name.to_string()))?;

            // Execute state change in widget
            widget.state_changed(key, &state);

            // Generate a list of scene modifications triggered from this widget
            // XXX We could obivously optimize this more, if it's needed, and "send" only state mofications
            // that can be triggered by that state change.
            let widget_settings = self.widget_settings.get(widget_name).ok_or(
                DeviceModelError::WidgetSettingsNotFound(widget_name.to_string()),
            )?;
            for (entity, modifications) in &widget_settings.scene_modifications.config {
                for modification in modifications {
                    scene_changes.append(&mut widget.state_to_blender(entity, *modification));
                }
            }
        }
        Ok(scene_changes)
    }

    pub(crate) fn backend_state_changed(
        &mut self,
        topic: &str,
        payload: &str,
    ) -> Result<Vec<SceneModificationEvent>, DeviceModelError> {
        bevy::log::debug!("Received update: topic={} payload={}", topic, payload);

        let (item_name, state) = openhab::parse_open_hab_state(topic, payload)?;
        self.state_changed(item_name, &state)
    }

    pub fn get_items(&self) -> Vec<&String> {
        self.smart_home_items.keys().collect::<Vec<&String>>()
    }
}
