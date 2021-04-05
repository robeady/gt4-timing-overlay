use game_data::GameData;
use process_memory::{Architecture, Pid, ProcessHandleExt, TryIntoProcessHandle};
use ui::render_ui;
use window::App;

mod game_data;
mod processes;
mod ps2_types;
mod scan_memory;
mod ui;
mod window;

pub struct Locations {
    pub auto_nan_offsets: Vec<usize>,
}

fn main() {
    let pid = processes::get_pcsx2_process_id();
    let handle = (pid as Pid)
        .try_into_process_handle()
        .unwrap()
        .set_arch(Architecture::Arch32Bit);

    let mut game_data = GameData::connect(handle);

    let window_size = [400.0, 300.0];
    let app = App::init("GT4 timing", window_size);
    app.main_loop(
        move |ui| render_ui(ui, window_size, &mut game_data, false),
        || {},
    );
}
