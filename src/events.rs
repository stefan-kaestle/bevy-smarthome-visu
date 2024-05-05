use bevy::prelude::*;
use serde::Serialize;

#[derive(Debug)]
pub(crate) enum SceneModificationEvent {
    LightModification(LightModification),
    SunModification(SunModification),
}

#[derive(Event, Debug)]
pub(crate) struct LightModification {
    pub(crate) entity_name: String,
    pub(crate) illuminance_percentage: f32,
}

#[derive(Event, Debug, Serialize, PartialEq)]
pub struct SunModification {
    pub(crate) illuminance: f64,
    pub(crate) elevation: f32,
    pub(crate) azimuth: f32,
}
