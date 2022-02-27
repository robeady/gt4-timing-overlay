use sysinfo::{ProcessExt, SystemExt};

pub fn get_pcsx2_process_id() -> usize {
    let mut system = sysinfo::System::new_all();
    system.refresh_processes();
    let candidates = system.get_process_by_name("pcsx2");
    if candidates.len() == 0 {
        panic!("no pcsx2 process found")
    } else {
        return candidates[0].pid();
    }
}
