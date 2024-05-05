use crate::openhab::RequestedStateChangeFromWidget;

#[derive(Debug, PartialEq)]
pub enum DeviceModelError {
    WidgetNotFound(String),
    ItemNotFound(String),
    ParserError(String),
    WidgetSettingsNotFound(String),
    ViewNotFound(String),
    KeyNotFound(RequestedStateChangeFromWidget),
    NoViewSelected(()),
}
