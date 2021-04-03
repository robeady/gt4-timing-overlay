use game_data::GameData;
use process_memory::{Architecture, Pid, ProcessHandleExt, TryIntoProcessHandle};

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

    let gd = GameData::connect(handle);
    ui::render_window(gd);
}
