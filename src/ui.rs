use imgui::*;

use crate::{game_data::GameData, ps2_types::Ps2Memory, window::App};

pub fn render_window<M: Ps2Memory + 'static>(
    mut game_data: GameData<M>,
    before_exit: impl Fn() + 'static,
) {
    let window_size = [400.0, 300.0];
    let app = App::init("GT4 timing", window_size);
    app.main_loop(
        move |ui| {
            let cars = game_data.read_cars();
            let entries = game_data.read_entries();

            game_data.sample_car_checkpoints();

            let track_length = game_data.read_track_length();

            let styles = ui.push_style_var(StyleVar::WindowRounding(0f32));

            Window::new(im_str!("GT4 Timing"))
                .title_bar(false)
                .resizable(false)
                .movable(false)
                .position([0f32, 0f32], Condition::Appearing)
                .size(window_size, Condition::Appearing)
                .build(ui, || {
                    ui.text(im_str!("Hello world!"));
                    ui.text(im_str!("track is {:.3}km long", track_length / 1000.0));
                    ui.separator();
                    for i in 0..6 {
                        let mass = cars[i].car_spec.get(&game_data.ps2).mass;
                        let name: String = entries[i].car_name_short.into();
                        let gap_to_leader =
                            game_data.gap_to_leader_ms(i).unwrap_or(f32::NAN) / 1000f32;
                        ui.text(format!(
                            "+{:.1} {}  ({:.1} {:.0}kg)",
                            gap_to_leader, name, cars[i].meters_driven_in_current_lap, mass
                        ))
                    }
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(format!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                    ));
                });

            styles.pop(&ui);
        },
        before_exit,
    );
}
