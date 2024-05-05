use hsl::HSL;
use serde::{Deserialize, Serialize};

/// State change requested from widget.
///
/// This contains the key as visible to the widget.
pub enum WidgetInteraction {
    StateChange(RequestedStateChangeFromWidget),
    FullscreenRequest(bool),
}

#[derive(Debug, PartialEq, Clone)]
pub struct RequestedStateChangeFromWidget {
    pub key: String,
    pub value: String,
}

/// State change request to be appended for sending over the network.
///
/// This has the key looked up to an item in the backend.
pub struct RequestedStateChange {
    pub item: String,
    pub value: String,
}

use crate::{errors::DeviceModelError, widget_settings::WidgetItemList};

impl RequestedStateChange {
    pub(crate) fn from_widget_request(
        widget_request: &RequestedStateChangeFromWidget,
        item_list: &WidgetItemList,
    ) -> Result<Self, DeviceModelError> {
        for (key, item) in &item_list.items {
            if key == &widget_request.key {
                return Ok(Self {
                    item: item.to_string(),
                    value: widget_request.value.clone(),
                });
            }
        }
        Err(DeviceModelError::KeyNotFound(widget_request.clone()))
    }
}

#[derive(Deserialize, Debug)]
pub struct OpenHabPayload {
    #[serde(rename = "type")]
    pub ohtype: String,
    pub value: String,
}

#[derive(Deserialize, Debug)]
pub struct OpenHabSimplePayload {
    pub status: String,
    #[serde(rename = "statusDetail")]
    pub status_detail: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OpenHabState {
    pub topic: String,
    pub payload: String,
    #[serde(rename = "type")]
    pub ohtype: Option<String>,
}

pub fn openhab_hsb_to_rgb(hsb: [f32; 3]) -> [f32; 3] {
    let rgb = HSL {
        h: hsb[0] as f64,
        s: hsb[1] as f64 / 100.,
        l: hsb[2] as f64 / 200.,
    }
    .to_rgb();

    return [
        rgb.0 as f32 / 255.,
        rgb.1 as f32 / 255.,
        rgb.2 as f32 / 255.,
    ];
}

fn parse_topic(topic: &str) -> Result<&str, DeviceModelError> {
    let tokens: Vec<&str> = topic.split("/").collect();
    if tokens.len() >= 4
        && tokens[0] == "openhab"
        && tokens[1] == "items"
        && (tokens[3] == "statechanged"
            || tokens[3] == "state"
            || tokens[3] == "stateupdated"
            || tokens[3] == "status")
    {
        return Ok(tokens[2]);
    }

    Err(DeviceModelError::ParserError(format!(
        "Failed to parse topic from {}",
        topic
    )))
}

#[test]
fn test_parse_topic() {
    assert_eq!(
        parse_topic("openhab/items/WashingMachinePower/state"),
        Ok("WashingMachinePower")
    );
}

/// Parse state update from OpenHab.
///
/// If parsing the state update is successful, the item and it's state are returned.
/// ERROR src/main.rs:498 Handling item state change failed: ParserError("Failed to parse topic=openhab/items/DeskPower/state, payload={\"type\":\"Quantity\",\"value\":\"130.75 W\"}") nextnext.js:474:21

pub(crate) fn parse_open_hab_state<'a>(
    topic: &'a str,
    payload: &'a str,
) -> Result<(&'a str, String), DeviceModelError> {
    // Try to parse as OpenHabPayload first
    let parsed_payload: serde_json::Result<OpenHabPayload> = serde_json::from_str(payload);
    match parsed_payload {
        Ok(t) => Ok((parse_topic(topic)?, t.value.clone())),
        Err(_) => {
            // If that fails, parse as OpenHabSimplePayload
            let parsed_payload: serde_json::Result<OpenHabSimplePayload> =
                serde_json::from_str(payload);

            let parsed_payload = parsed_payload.map_err(|e| {
                DeviceModelError::ParserError(format!(
                    "Failed to parse payload {} to OpenHabSimplePayload in parse_open_hab_state: {:?}", payload,
                    e
                ))
            })?;

            Ok((parse_topic(topic)?, parsed_payload.status.clone()))
        }
    }
}
