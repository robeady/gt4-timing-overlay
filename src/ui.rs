use imgui::*;
use std::cmp::Reverse;

use crate::{game_data::GameData, ps2_types::Ps2Memory};

pub fn init_ui(imgui: &mut imgui::Context, dpi_factor: f64) {
    let scaled_font_size = (32.0 * dpi_factor) as f32;
    imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../resources/texgyreheros-regular.ttf"),
        size_pixels: scaled_font_size,
        config: None,
    }]);
    imgui.io_mut().font_global_scale = (1.0 / dpi_factor) as f32;
}

pub fn render_ui<M: Ps2Memory>(
    ui: &Ui,
    window_size: [f32; 2],
    game_data: &mut GameData<M>,
    movable: bool,
    scale: f32,
) {
    let race_state = game_data.sample_race();

    let styles = ui.push_style_var(StyleVar::WindowRounding(0f32));
    let colors = ui.push_style_color(StyleColor::WindowBg, [0.0, 0.0, 0.0, 0.5]);

    Window::new(im_str!("Timing"))
        .title_bar(movable)
        .resizable(movable)
        .movable(movable)
        .position([0f32, 0f32], Condition::Appearing)
        .size(window_size, Condition::Appearing)
        .build(ui, || {
            if let Ok(r) = race_state {
                let mut sorted_car_indices: Vec<_> = (0..(r.cars.len())).collect();
                sorted_car_indices.sort_by_key(|&i| Reverse(r.cars[i].progress(r.track_length)));
                for i in sorted_car_indices {
                    let name: String = r.entries[i].car_name_short.into();
                    let gap_to_leader = r.gaps_to_leader[i].unwrap_or(f32::NAN) / 1000f32;
                    let text = im_str!(
                        "+{:.2} {} {}",
                        gap_to_leader,
                        ["F", "A", "B", "C", "D", "E"][i], // ugh maybe this assumes the player does not qualify
                        name
                    );
                    ui.text(text);
                }
            }
        });

    styles.pop(&ui);
    colors.pop(&ui);
}
