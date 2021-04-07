use imgui::*;

use crate::{game_data::GameData, ps2_types::Ps2Memory};

pub fn init_ui(imgui: &mut imgui::Context, dpi_factor: f64) {
    let scaled_font_size = (26.0 * dpi_factor) as f32;
    imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../resources/segoeui.ttf"),
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
    let colors = ui.push_style_color(StyleColor::WindowBg, [0.0, 0.0, 0.0, 0.0]);

    Window::new(im_str!("Timing"))
        .title_bar(false)
        .resizable(false)
        .movable(movable)
        .position([0f32, 0f32], Condition::Appearing)
        .size(window_size, Condition::Appearing)
        .build(ui, || {
            if let Ok(r) = race_state {
                ui.text(im_str!("track is {:.3}km long", r.track_length / 1000.0));
                ui.separator();
                for i in 0..(r.cars.len()) {
                    let mass =
                        r.cars[i].car_spec.get(&game_data.ps2).map(|c| c.mass).unwrap_or(f32::NAN);
                    let name: String = r.entries[i].car_name_short.into();
                    let gap_to_leader = r.gaps_to_leader[i].unwrap_or(f32::NAN) / 1000f32;
                    let text = im_str!(
                        "+{:.1} {}  ({:.1} {:.0}kg)",
                        gap_to_leader,
                        name,
                        r.cars[i].meters_driven_in_current_lap,
                        mass
                    );

                    let [text_width, text_height] = ui.calc_text_size(&text, false, f32::INFINITY);
                    let [cursor_x, cursor_y] = ui.cursor_screen_pos();
                    ui.get_window_draw_list()
                        .add_rect(
                            [cursor_x, cursor_y],
                            [cursor_x + text_width, cursor_y + text_height],
                            [0.0, 0.0, 0.0, 0.4],
                        )
                        .filled(true)
                        .build();

                    ui.text(text);
                }
                ui.separator();
                let mouse_pos = ui.io().mouse_pos;
                ui.text(format!("Mouse Position: ({:.1},{:.1})", mouse_pos[0], mouse_pos[1]));
            }
        });

    styles.pop(&ui);
    colors.pop(&ui);
}
