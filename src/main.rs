use game_data::GameData;
use log::LevelFilter;
use process_memory::{Architecture, Pid, ProcessHandleExt, TryIntoProcessHandle};
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode, ThreadLogMode};
use ui::{init_ui, render_ui};
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
    let log_config = ConfigBuilder::new()
        .set_thread_level(LevelFilter::Error)
        .set_thread_mode(ThreadLogMode::Both)
        .build();
    TermLogger::init(
        LevelFilter::Debug,
        log_config.clone(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    let pid = processes::get_pcsx2_process_id();
    let handle = (pid as Pid).try_into_process_handle().unwrap().set_arch(Architecture::Arch32Bit);

    let mut game_data = GameData::connect(handle);

    let window_size = [400.0, 300.0];
    let mut app = App::init("GT4 timing", window_size);
    init_ui(&mut app.imgui, app.dpi_factor);
    app.main_loop(move |ui| render_ui(ui, window_size, &mut game_data, false, 1.0), || {});
}
