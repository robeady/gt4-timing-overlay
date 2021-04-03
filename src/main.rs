use process_memory::{Architecture, Pid, ProcessHandleExt, TryIntoProcessHandle};

mod automobile;
mod processes;
mod scan;

fn main() {
    let pid = processes::get_pcsx2_process_id();
    let handle = (pid as Pid)
        .try_into_process_handle()
        .unwrap()
        .set_arch(Architecture::Arch32Bit);

    let mut sig = Vec::new();
    sig.extend_from_slice(&[0xFF; 64]);
    sig.extend_from_slice(&[0xA3, 0x70, 0x7D, 0x3F]);
    let offsets = scan::find_all_offsets(&sig, handle);
    println!("{:?}", offsets);

    automobile::read_automobiles(offsets[0], handle);

    // find memory locations of magic numbers
    // calculate memory location of data block
    // every second, re-read values and calculate gaps
}
