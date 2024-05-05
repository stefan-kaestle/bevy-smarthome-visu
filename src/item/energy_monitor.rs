use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error;
use crate::item::generic_item::{generic_translate_value, GenericItem};

// Number of highest power meter values to take for display.
const NUM_HIGHEST: usize = 3;
const DEFAULT_VOLTAGE: usize = 230;

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct PowerMeter {
    pub name: String,
    pub label: String,
    pub unit: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnergyMonitorConfig {
    pub power_meters: Vec<PowerMeter>,
}

fn translate_value(key: &str, input: &str) -> Value {
    if let Ok(p) = input.parse::<f32>() {
        if key.ends_with("strom") {
            Value::String(format!("{:.2}", p))
        } else {
            Value::String(format!("{:.1}", p))
        }
    } else {
        generic_translate_value(key, input)
    }
}

fn parse_power_meter_to_watts(input: &str, unit: Option<&str>) -> f32 {
    let tokens: Vec<&str> = input.split(" ").collect();
    let value = tokens.get(0).map(|s| s.parse::<f32>());
    match tokens.get(1) {
        Some(&"W") => value.unwrap().unwrap(),
        Some(unknown_unit) => {
            error!(
                "Failed to parse unknown unit for power meter: {}",
                unknown_unit
            );
            0.0
        }
        None => match unit {
            Some("mA") | Some("miliampere") => {
                value.unwrap().unwrap() * DEFAULT_VOLTAGE as f32 / 1000.
            }
            Some(_) | None => value.unwrap().unwrap(),
        },
    }
}

fn init_map(generic_item: &GenericItem<EnergyMonitorConfig>) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();

    // XXX That's not the energy, it's the power!
    let mut energy_total = 0.0;
    for i in 1..4 {
        let parsed = generic_item
            .state
            .get(&format!("l{}_strom", i))
            .map(|s| s.0.parse::<f32>());
        let (energy_as_str, energy) = match parsed {
            None => ("n.a.".to_string(), 0.0),
            Some(s) => match s {
                Ok(s) => (format!("{:.1}", s * 230.0), s * 230.0),
                Err(_) => ("n.a.".to_string(), 0.0),
            },
        };
        map.insert(
            format!("l{}_energy_calculated", i),
            Value::String(energy_as_str),
        );
        energy_total += energy;
    }

    // Power value for configured power meters
    let mut power_meters = vec![];
    // Need this for sorting
    let mut power_meter_values = vec![];
    let mut power_meter_totals = 0.0;
    if let Some(config) = &generic_item.config {
        for power_meter in &config.power_meters {
            let state = generic_item.state.get(&power_meter.name);
            if let Some(state) = state {
                let parsed = parse_power_meter_to_watts(
                    &state.0,
                    power_meter.unit.as_ref().map(|s| s.as_str()),
                );
                if parsed > 0. {
                    let formatted = format!("{:.0} W", parsed);
                    power_meters.push((power_meter.label.to_string(), formatted.clone()));
                    power_meter_values.push((power_meter.label.to_string(), parsed, formatted));
                    power_meter_totals += parsed;
                }
            }
        }
    }

    // Calculate the remainder of the power consumption and add it to the list
    let misc_power = energy_total - power_meter_totals;
    map.insert(
        "misc_power".to_string(),
        Value::String(format!("{:.1} W", misc_power)),
    );
    power_meter_values.push((
        "Misc".to_string(),
        misc_power,
        format!("{:.1} W", misc_power),
    ));

    // Sort power meter values and take the NUM_HIGHEST highest ones.
    power_meter_values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    map.insert(
        "power_meters".to_string(),
        serde_json::to_value(power_meters).unwrap(),
    );
    map.insert(
        "power_meter_totals".to_string(),
        Value::String(format!("{:.1} W", power_meter_totals)),
    );

    let mut power_meter_highest = vec![];
    for (item_label, _, value_formatted) in power_meter_values.iter().take(NUM_HIGHEST) {
        power_meter_highest.push((item_label, value_formatted));
    }

    map.insert(
        "power_meter_highest".to_string(),
        serde_json::to_value(power_meter_highest).unwrap(),
    );

    map.insert(
        "energy_total".to_string(),
        Value::String(format!("{:.1}", energy_total)),
    );
    map
}

fn render_slider(generic_item: &GenericItem<EnergyMonitorConfig>) -> Option<(String, usize)> {
    let val = generic_item.state.get("leistung")?.0.parse::<f32>().ok()?;
    if val > 700.0 {
        Some(("slider-energy-monitor".to_string(), 15))
    } else {
        None
    }
}

pub fn new() -> GenericItem<EnergyMonitorConfig> {
    GenericItem::with_custom_functions(translate_value, init_map, render_slider)
}
