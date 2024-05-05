use bevy::log::info;
use bevy_egui::egui::{
    self, emath::RectTransform, Color32, Image, ImageButton, LayerId, Pos2, Rect, RichText,
    Rounding, Shape, Vec2,
};

use crate::{
    openhab::{RequestedStateChangeFromWidget, WidgetInteraction},
    widget_settings::WidgetRenderSetting,
};

const TINT_OFF: egui::Color32 = egui::Color32::GRAY;
const ICON_SCALE: Vec2 = Vec2::splat(2.0);

const MARGIN: f32 = 4.;
const ROUNDING: f32 = 4.;

pub(crate) const DARK_GREEN: Color32 = Color32::from_rgb(79, 200, 114);
pub(crate) const DARK_YELLOW: Color32 = Color32::from_rgb(79, 200, 114);

pub fn render_simple_button(
    ctx: &mut egui::Context,
    render_setting: &WidgetRenderSetting,
    render_position: (f32, f32),
    state: bool,
    watts: Option<f32>,
    icon: Image,
    on_color: Color32,
    on_command: Option<&str>,
    off_command: Option<&str>,
) -> Vec<WidgetInteraction> {
    let mut request = vec![];

    ctx.style_mut(|style| {
        // In case we want to change the styling
    });

    egui::Area::new(render_setting.id)
        .fixed_pos(egui::pos2(render_position.0, render_position.1))
        .show(ctx, |ui| {
            let response = ui.horizontal(|ui| {
                if state {
                    let widget = ui.add(
                        ImageButton::new(icon.clone().fit_to_fraction(ICON_SCALE))
                            .rounding(ROUNDING),
                    );
                    if widget.clicked() {
                        request.push(WidgetInteraction::StateChange(
                            RequestedStateChangeFromWidget {
                                key: render_setting.widget_name.to_string(),
                                value: off_command.unwrap_or("OFF").to_string(),
                            },
                        ));
                    };
                    if widget.secondary_clicked() {
                        request.push(WidgetInteraction::FullscreenRequest(true));
                    }
                } else {
                    let widget = ui.add(
                        ImageButton::new(icon.clone().fit_to_fraction(ICON_SCALE).tint(TINT_OFF))
                            .rounding(ROUNDING),
                    );
                    if widget.clicked() {
                        request.push(WidgetInteraction::StateChange(
                            RequestedStateChangeFromWidget {
                                key: render_setting.widget_name.to_string(),
                                value: on_command.unwrap_or("ON").to_string(),
                            },
                        ));
                    };
                    if widget.secondary_clicked() {
                        request.push(WidgetInteraction::FullscreenRequest(true));
                    }
                }

                if let Some(label) = &render_setting.label {
                    ui.label(label);
                }

                if state {
                    if let Some(watts) = watts {
                        ui.label(RichText::new(format!("{}W", watts)).small());
                    }
                }
            });

            // This response contains the actual size of the horizontal().
            let response = response.response;

            // Get the relative position of our "canvas"
            let to_screen = RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, response.rect.size()),
                response.rect,
            );

            // The line we want to draw represented as 2 points
            let first_point = Pos2 { x: 0.0, y: 0.0 };
            let second_point = Pos2 {
                x: response.rect.width(),
                y: response.rect.height(),
            };
            // Make the points relative to the "canvas"
            let first_point_in_screen = to_screen.transform_pos(first_point);
            let second_point_in_screen = to_screen.transform_pos(second_point);

            // Paint the line!
            ui.with_layer_id(LayerId::background(), |ui| {
                ui.painter().add(Shape::rect_filled(
                    Rect::from_two_pos(
                        Pos2 {
                            x: first_point_in_screen.x - MARGIN,
                            y: first_point_in_screen.y - MARGIN,
                        },
                        Pos2 {
                            x: second_point_in_screen.x + MARGIN,
                            y: second_point_in_screen.y + MARGIN,
                        },
                    ),
                    Rounding {
                        nw: ROUNDING,
                        ne: ROUNDING,
                        sw: ROUNDING,
                        se: ROUNDING,
                    },
                    if state { on_color } else { Color32::BLACK },
                ))
            });
        });

    request
}
