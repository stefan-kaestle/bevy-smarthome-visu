use bevy_egui::egui::Image;

pub(crate) const LIGHT_BULB: &str = "1F4A1";

pub(crate) fn get_emoji(code: &str) -> Image {
    match code {
        // Light bulb
        "1F4A1" => Image::from_bytes(
            "bytes://1F4A1.png",
            include_bytes!("../assets/emoji/1F4A1.png"),
        ),
        //  Coffee machine
        "E150" => Image::from_bytes(
            "bytes://E150.png",
            include_bytes!("../assets/emoji/E150.png"),
        ),
        _ => panic!("Could not find image {}", code),
    }
}
