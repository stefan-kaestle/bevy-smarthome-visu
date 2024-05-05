use bevy::utils::HashMap;
use bevy_egui::egui;

#[derive(Debug)]
pub struct WidgetRenderSetting {
    /// Top coordinate for rendering on the screen.
    pub(crate) top: f32,
    /// Left coordinate for redering on the screen.
    pub(crate) left: f32,
    /// An ID to be used by egui
    pub(crate) id: egui::Id,
    /// On optional label string. Depending on the widget, this might or might not be rendered.
    pub(crate) label: Option<String>,
    /// The name of the widget
    pub(crate) widget_name: String,
}

impl WidgetRenderSetting {
    /// Initialize a render setting instance from the widget configuration
    pub(crate) fn from_widget_config(
        view_name: &str,
        widget_name: &str,
        widget_config: &crate::config::Item,
    ) -> Option<Self> {
        if let Some(top) = &widget_config.top {
            if let Some(left) = &widget_config.left {
                let top = top.replace("%", "").parse::<f32>().expect(&format!(
                    "Fafiled to parse top coordinate for {}",
                    widget_name
                )) * -1.;
                let left = left.replace("%", "").parse::<f32>().expect(&format!(
                    "Fafiled to parse left coordinate for {}",
                    widget_name
                ));

                let id = egui::Id::new(format!("{}_{}", view_name, widget_name));

                return Some(WidgetRenderSetting {
                    top,
                    left,
                    id,
                    label: widget_config.label.clone(),
                    widget_name: widget_name.to_string(),
                });
            }
        }
        None
    }
}

/// The name of a widget.
pub(crate) type WidgetName = String;
/// Multiple smart home items can be assigned to the same widget. Internally,
/// those are distinguished by means of this WidgetKey.
pub(crate) type WidgetKey = String;

/// The name of a view.
pub(crate) type ViewName = String;

/// The name of an item. Should match the name in backend.
pub(crate) type ItemName = String;

/// A string representing an entityt name in the scene.
pub(crate) type EntityName = String;

/// A list of items assigned to a widget.
pub(crate) struct WidgetItemList {
    /// Each entry is a tuple of (key, item).
    /// This data structure is used to figure out which item name is used in the backend when
    /// an action is triggered on an item. The widget needs to work out which item is
    /// attached to each key.
    pub(crate) items: Vec<(WidgetKey, ItemName)>,
}

impl WidgetItemList {
    /// Initialize a render setting instance from the widget configuration
    ///
    /// The list of items is always at least one, i.e. there cannot be a widget without any
    /// smart home items.
    ///
    /// If no smart home item is explicitly given, we assume that the item has the same name
    /// as the widget (convenience and backward compatibility).
    pub(crate) fn from_widget_config(
        widget_name: &str,
        widget_config: &crate::config::Item,
    ) -> Self {
        let mut items = vec![];
        for (item_name, value) in &widget_config.smarthome_items {
            let key: WidgetKey = value.key.as_ref().unwrap_or(item_name).to_string();
            items.push((key, item_name.to_string()));
        }
        // No item explictly given, so we assume item name equals widget name.
        if items.len() < 1 {
            items.push((widget_name.to_string(), widget_name.to_string()));
        }
        Self { items }
    }
}

// Enum expressing the types of scene modifications we support
#[derive(Clone, Copy, Debug)]
pub(crate) enum SceneModification {
    Energy(i32),
    Color(),
    Array(),
    Sun(),
}

const MAX_ILLUMINATION: i32 = 400;

/// Map string representation of scenen modification to corresponding enum
impl SceneModification {
    pub(crate) fn from_widget_config(string_representation: &str) -> Self {
        match string_representation {
            "Energy" => SceneModification::Energy(MAX_ILLUMINATION),
            "Color" => SceneModification::Color(),
            "Array" => SceneModification::Array(),
            "Sun" => SceneModification::Sun(),
            _ => {
                panic!(
                    "Unsupported scene configuration {} found in config",
                    &string_representation
                )
            }
        }
    }
}

pub(crate) struct WidgetSceneModifications {
    pub(crate) config: HashMap<EntityName, Vec<SceneModification>>,
}

impl WidgetSceneModifications {
    /// Parse scene modifications part of the widget configuration.
    pub(crate) fn from_widget_config(
        config: &std::collections::HashMap<String, Vec<String>>,
    ) -> Self {
        let mut hm = HashMap::new();
        for (entity, modifications) in config {
            hm.insert(
                entity.to_string(),
                modifications
                    .iter()
                    .map(|s| SceneModification::from_widget_config(s))
                    .collect::<Vec<SceneModification>>(),
            );
        }
        Self { config: hm }
    }
}

pub(crate) struct WidgetSettings {
    /// Settings used to render the widget
    pub(crate) render_settings: Option<WidgetRenderSetting>,
    // Mappings of backend items to keys in the widget.
    pub(crate) item_list: WidgetItemList,
    // Changes to the 3D scene
    pub(crate) scene_modifications: WidgetSceneModifications,
}
