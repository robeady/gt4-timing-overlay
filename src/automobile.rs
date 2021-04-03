use derivative::Derivative;
use process_memory::{DataMember, Memory, ProcessHandle};
use std::mem::size_of;

// 4256 bytes
// each struct starts a bit before the NaN block, guess 50 bytes
#[repr(C, packed(1))]
#[derive(Derivative, Clone, Copy)]
#[derivative(Debug)]
struct Automobile {
    #[derivative(Debug = "ignore")]
    _unk0: [u8; 50],
    #[derivative(Debug = "ignore")]
    nans: [u8; 64],
    #[derivative(Debug = "ignore")]
    _unk1276: [u8; 1216],
    throttle1: f32, // + 1280
    brake1: f32,    // + 1284
    #[derivative(Debug = "ignore")]
    _unk4: [u8; 4],
    throttle2: f32, // + 1292
    #[derivative(Debug = "ignore")]
    _unk100: [u8; 12],
    meters_driven_in_current_lap: f32,
    #[derivative(Debug = "ignore")]
    _unkz: [u8; 4],
    implicit_current_lap: u16, // from decompiler
    #[derivative(Debug = "ignore")]
    _unk100_2: [u8; 78],
    almost_rpm: f32, // + 1396
    #[derivative(Debug = "ignore")]
    _unk32: [u8; 32],
    gear: u8, // + 1432
    #[derivative(Debug = "ignore")]
    _unk274: [u8; 275],
    rpm: f32, // + 1708
    #[derivative(Debug = "ignore")]
    _unk12: [u8; 12],
    throttle3: f32, // + 1724
    #[derivative(Debug = "ignore")]
    _unk52: [u8; 52],
    throttle4: f32, // + 1780
    brake2: f32,    // + 1784
    #[derivative(Debug = "ignore")]
    _unk: [u8; 2418],
}

const BEFORE_NANS: usize = 50;
const STRUCT_SIZE: usize = 4256;

pub fn read_automobiles(first_nan_offset: usize, handle: ProcessHandle) {
    if size_of::<Automobile>() != STRUCT_SIZE {
        panic!(
            "wrong size, got {}, expected {}",
            size_of::<Automobile>(),
            STRUCT_SIZE
        )
    }

    let array_offset = first_nan_offset - BEFORE_NANS - STRUCT_SIZE;
    let member = DataMember::<[Automobile; 6]>::new_offset(handle, vec![array_offset]);

    println!("{:#?}", member.read().unwrap());
}
