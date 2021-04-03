use crate::scan_memory;
use derivative::Derivative;
use process_memory::{DataMember, Memory, ProcessHandle};

const BEFORE_NANS: usize = 50;
const STRUCT_SIZE: usize = 4256;

// 4256 bytes
// each struct starts a bit before the NaN block, guess 50 bytes
#[repr(C, packed(1))]
#[derive(Derivative, Clone, Copy)]
#[derivative(Debug)]
pub struct Automobile {
    #[derivative(Debug = "ignore")]
    _unk0: [u8; BEFORE_NANS],
    #[derivative(Debug = "ignore")]
    nans: [u8; 64],
    #[derivative(Debug = "ignore")]
    _unk1276: [u8; 1216],
    pub throttle1: f32, // + 1280
    pub brake1: f32,    // + 1284
    #[derivative(Debug = "ignore")]
    _unk4: [u8; 4],
    pub throttle2: f32, // + 1292
    #[derivative(Debug = "ignore")]
    _unk100: [u8; 12],
    pub meters_driven_in_current_lap: f32,
    #[derivative(Debug = "ignore")]
    _unkz: [u8; 4],
    implicit_current_lap: u16, // from decompiler
    #[derivative(Debug = "ignore")]
    _unk100_2: [u8; 78],
    pub almost_rpm: f32, // + 1396
    #[derivative(Debug = "ignore")]
    _unk32: [u8; 32],
    gear: u8, // + 1432
    #[derivative(Debug = "ignore")]
    _unk274: [u8; 275],
    pub rpm: f32, // + 1708
    #[derivative(Debug = "ignore")]
    _unk12: [u8; 12],
    pub throttle3: f32, // + 1724
    #[derivative(Debug = "ignore")]
    _unk52: [u8; 52],
    pub throttle4: f32, // + 1780
    pub brake2: f32,    // + 1784
    #[derivative(Debug = "ignore")]
    _unk: [u8; 2418],
}

static_assertions::assert_eq_size!([u8; STRUCT_SIZE], Automobile);

pub struct GameData {
    process_handle: ProcessHandle,
    auto_nan_offsets: Vec<usize>,
}

impl GameData {
    pub fn connect(process_handle: ProcessHandle) -> GameData {
        println!("Finding autos");
        let mut sig = Vec::new();
        sig.extend_from_slice(&[0xFF; 64]);
        sig.extend_from_slice(&[0xA3, 0x70, 0x7D, 0x3F]);
        let offsets = scan_memory::find_all_offsets(&sig, process_handle);
        println!("Found autos at {:?}", offsets);
        return GameData {
            process_handle,
            auto_nan_offsets: offsets,
        };
    }

    pub fn read_autos(&self) -> Vec<Automobile> {
        let array_offset = self.auto_nan_offsets[0] - BEFORE_NANS - STRUCT_SIZE;
        let member =
            DataMember::<[Automobile; 6]>::new_offset(self.process_handle, vec![array_offset]);
        return member.read().unwrap().to_vec();
    }
}
