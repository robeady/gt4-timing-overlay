use sysinfo::{ProcessExt, SystemExt};

pub fn get_pcsx2_process_id() -> usize {
    let mut system = sysinfo::System::new_all();
    system.refresh_processes();
    return system.get_process_by_name("pcsx2")[0].pid();
}
