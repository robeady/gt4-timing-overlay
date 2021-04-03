use imgui::*;

use crate::{
    game_data::{Automobile, Entry, GameData},
    window::App,
};

pub fn render_window(game_data: GameData) {
    let window_size = [400.0, 300.0];
    let app = App::init("GT4 timing", window_size);
    app.main_loop(move |_, ui| {
        let mut autos: Vec<Automobile> = Vec::new();
        let mut entries: Vec<Entry> = Vec::new();
        autos = game_data.read_autos();
        entries = game_data.read_entries();

        let styles = ui.push_style_var(StyleVar::WindowRounding(0f32));

        Window::new(im_str!("GT4 Timing"))
            .title_bar(false)
            .resizable(false)
            .movable(false)
            .position([0f32, 0f32], Condition::Appearing)
            .size(window_size, Condition::Appearing)
            .build(ui, || {
                ui.text(im_str!("Hello world!"));
                ui.separator();
                for i in 0..6 {
                    let mass = autos[i].car_spec.get(&game_data.ps2).mass;
                    let name: String = entries[i].car_name_short.into();
                    ui.text(format!(
                        "{} - {:.1} {:.0}kg",
                        name, autos[i].meters_driven_in_current_lap, mass
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
    });
}
