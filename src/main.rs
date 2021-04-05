use game_data::GameData;
use process_memory::{Architecture, Pid, ProcessHandleExt, TryIntoProcessHandle};
use winapi::shared::minwindef::HMODULE;

mod game_data;
mod inject;
mod processes;
mod ps2_types;
mod scan_memory;
mod ui;
mod window;

pub struct Locations {
    pub auto_nan_offsets: Vec<usize>,
}

fn main() {
    env_logger::init();

    let pid = processes::get_pcsx2_process_id();
    let handle = (pid as Pid)
        .try_into_process_handle()
        .unwrap()
        .set_arch(Architecture::Arch32Bit);

    let dll = "target/i686-pc-windows-msvc/release/timing_lib.dll";

    let hmodule = dbg!(inject::inject(pid as u32, dll).unwrap());

    let gd = GameData::connect(handle);
    ui::render_window(gd, move || {
        let free_library_ret_val = inject::uninject(pid as u32, hmodule).unwrap();
        println!("{}", free_library_ret_val);
    });
}
