use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::item::generic_item::{generic_translate_value, GenericItem};

use super::NotificationStatus;

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContactConfig {}

macro_rules! log {
    ( $( $t:tt )* ) => {};
}

fn is_closed(generic_item: &GenericItem<ContactConfig>) -> bool {
    // Contacts should really only have a single key.
    match generic_item.state.iter().next() {
        Some((_key, (value, _time))) => value == "CLOSED",
        None => true,
    }
}

fn init_map(generic_item: &GenericItem<ContactConfig>) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    map.insert(
        "is_closed".to_string(),
        Value::Bool(is_closed(generic_item)),
    );
    map
}

fn render_slider(generic_item: &GenericItem<ContactConfig>) -> Option<(String, usize)> {
    if !is_closed(generic_item) {
        Some(("slider-door".to_string(), 200))
    } else {
        None
    }
}

// fn state_to_blender(
//     generic_item: &GenericItem<ContactConfig>,
//     _blender_item: &str,
//     modification: &str,
//     _max_energy: f32,
// ) -> Vec<BlenderConf> {
//     let val = is_closed(generic_item);
//     match modification {
//         "Hidden" => vec![BlenderConf::Hidden(BlenderConfHidden { val: !val })],
//         "Show" => vec![BlenderConf::Hidden(BlenderConfHidden { val })],
//         _ => {
//             log!(
//                 "Received unknown blender modification request: {}",
//                 modification
//             );
//             vec![]
//         }
//     }
// }

fn get_notification_status(
    generic_item: &GenericItem<ContactConfig>,
) -> Option<NotificationStatus> {
    if !is_closed(generic_item) {
        Some(NotificationStatus {
            color: "red".to_string(),
            priority: 2,
            num: 1,
        })
    } else {
        None
    }
}

pub(crate) fn new() -> GenericItem<ContactConfig> {
    let mut d =
        GenericItem::with_custom_functions(generic_translate_value, init_map, render_slider);
    d.with_notification_f(get_notification_status);
    d
}

// fn test_contact() {
//     use crate::conf::{ConfigBuilder, ItemBuilder};
//     use crate::DeviceModel;
//     let mut device = DeviceModel::with_configuration(
//         &ConfigBuilder::new()
//             .add_item(
//                 "sample_contact",
//                 ItemBuilder::new("Contact")
//                     .with_template("basic-door")
//                     .add_blender_item("blender_window_closed", vec!["Hidden"])
//                     .add_blender_item("blender_window_open", vec!["Show"])
//                     .build(),
//             )
//             .to_str(),
//         "http://127.0.0.1".to_string(),
//         "".to_string(),
//         "".to_string(),
//         false,
//     );

//     // TODO: Use get_key_from_state
//     let is_closed = |d: &DeviceModel| {
//         let state_as_string = d.get_state_as_json("sample_contact");
//         let state_as_json: serde_json::Value = serde_json::from_str(&state_as_string).expect(
//             &format!("Failed to parse string {:?} as json", &state_as_string),
//         );
//         state_as_json.get("is_closed").unwrap().as_bool().unwrap()
//     };

//     device.set_item_state("sample_contact", "OPEN", false);
//     assert!(!is_closed(&device));

//     assert_eq!(
//         device
//             .item_list
//             .read()
//             .unwrap()
//             .items
//             .get("sample_contact")
//             .unwrap()
//             .state_to_blender("blender_window_open", "Show", 0.0),
//         vec![BlenderConf::Hidden(BlenderConfHidden { val: false })]
//     );
//     assert_eq!(
//         device
//             .item_list
//             .read()
//             .unwrap()
//             .items
//             .get("sample_contact")
//             .unwrap()
//             .state_to_blender("blender_window_closed", "Hidden", 0.0),
//         vec![BlenderConf::Hidden(BlenderConfHidden { val: true })]
//     );

//     device.set_item_state("sample_contact", "CLOSED", false);
//     assert!(is_closed(&device));

//     // Check timestamp of the last state change has been added.
//     let state_as_string = device.get_state_as_json("sample_contact");
//     let state_as_json: serde_json::Value = serde_json::from_str(&state_as_string).expect(&format!(
//         "Failed to parse string {:?} as json",
//         &state_as_string
//     ));
//     assert!(state_as_json
//         .get("__has_last_updated")
//         .unwrap()
//         .as_bool()
//         .unwrap());
//     assert!(state_as_json.get("__last_updated").is_some());

//     assert_eq!(
//         device
//             .item_list
//             .read()
//             .unwrap()
//             .items
//             .get("sample_contact")
//             .unwrap()
//             .state_to_blender("blender_window_open", "Show", 0.0),
//         vec![BlenderConf::Hidden(BlenderConfHidden { val: true })]
//     );
//     assert_eq!(
//         device
//             .item_list
//             .read()
//             .unwrap()
//             .items
//             .get("sample_contact")
//             .unwrap()
//             .state_to_blender("blender_window_closed", "Hidden", 0.0),
//         vec![BlenderConf::Hidden(BlenderConfHidden { val: false })]
//     );
// }
