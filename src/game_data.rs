use std::mem::size_of;

use crate::{
    ps2_types::{Ps2Memory, Ps2Ptr, Ps2String},
    scan_memory,
};
use derivative::Derivative;
use process_memory::{DataMember, Memory, ProcessHandle};

const BEFORE_NANS: usize = 140;

// 4256 bytes
// each struct starts a bit before the NaN block, guess 50 bytes
#[repr(C, packed(1))]
#[derive(Derivative, Clone, Copy)]
#[derivative(Debug)]
pub struct Automobile {
    pub race_organisation: Ps2Ptr<()>,
    pub dynamics_conductor: Ps2Ptr<()>,
    _unkptr1: Ps2Ptr<()>,
    _unkptr2: Ps2Ptr<()>,
    pub car_spec: Ps2Ptr<CarSpec>,
    #[derivative(Debug = "ignore")]
    _unk0: [u8; BEFORE_NANS - 20],
    #[derivative(Debug = "ignore")]
    nans: [u8; 64], // + 0
    #[derivative(Debug = "ignore")]
    _unk1276: [u8; 1216],
    pub throttle_pedal: f32, // + 1280
    pub brake1: f32,         // + 1284
    #[derivative(Debug = "ignore")]
    _unk4: [u8; 4],
    pub throttle_actual: f32, // + 1292
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
    _unk: [u8; 2328],
}

static_assertions::assert_eq_size!([u8; 4256], Automobile);

// size unknown, at least 5500 bytes or so
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CarSpec {
    unknown: [u8; 72],
    pub mass: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Entry {
    // 68 tuning ints, I think each entry is a pair (row id, table id)
    pub tuning_data: [TuningItem; 34],
    pub _unk0: [u8; 132],
    pub timing_data: [i32; 1038],
    pub _unk1: [u8; 8720],
    pub engine_sound_path: [u8; 32],
    pub normal_sound_path: [u8; 32],
    pub _unk2: [u8; 80],
    pub car_name_short: Ps2String<128>,
    pub car_name: Ps2String<192>,
    pub _unk3: [u8; 52],
}

static_assertions::assert_eq_size!([u8; 13792], Entry);

impl Entry {
    pub fn tuning(&self, table: TuningTable) -> Option<i32> {
        for item in self.tuning_data.iter() {
            if item.table == table {
                return Some(item.row_id);
            }
        }
        return None;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TuningItem {
    pub row_id: i32,
    pub table: TuningTable,
}

#[allow(non_camel_case_types)]
#[repr(i32)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TuningTable {
    GENERIC_CAR = 0,
    BRAKE = 1,
    CHASSIS = 6,
    RACING_MODIFY = 7,
    DRIVETRAIN = 10,
    GEARING = 11,
    ENGINE = 12,
    RACING_CHIP = 18,
    FRONT_TIRE = 25,
    REAR_TIRE = 26,
}

pub struct GameData {
    pub ps2: Ps2Memory,
}

// offset from EE main memory base to start of NaN block for automobiles[1]
const FIRST_NAN_OFFSET_FROM_EE_BASE: usize = 0x01C0EEA4;

impl GameData {
    const fn automobiles_start_address(&self) -> usize {
        return self.ps2.ee_base_address + FIRST_NAN_OFFSET_FROM_EE_BASE
            - BEFORE_NANS  // go to start of Automobile struct
            - size_of::<Automobile>(); // that was entry 1, go to entry 0
    }

    const fn entries_start_address(&self) -> usize {
        return self.ps2.ee_base_address + FIRST_NAN_OFFSET_FROM_EE_BASE - 0x2E0A4;
    }

    pub fn connect(process_handle: ProcessHandle) -> GameData {
        println!("Finding autos");
        let mut autos_sig = Vec::new();
        autos_sig.extend_from_slice(&[0xFF; 64]); // these are the NaNs
        autos_sig.extend_from_slice(&[0xA3, 0x70, 0x7D, 0x3F]);
        let offsets = scan_memory::find_all_offsets(&autos_sig, process_handle);
        println!("Found autos at {:?}", offsets);

        if offsets.len() != 5 {
            panic!(
                "found {} NaN blocks at {:?}, expected 5",
                offsets.len(),
                offsets
            )
        }

        let ee_base_address = offsets[0] - FIRST_NAN_OFFSET_FROM_EE_BASE;

        return GameData {
            ps2: Ps2Memory {
                ee_base_address,
                pcsx2_process_handle: process_handle,
            },
        };
    }

    pub fn read_autos(&self) -> Vec<Automobile> {
        let member = DataMember::<[Automobile; 6]>::new_offset(
            self.ps2.pcsx2_process_handle,
            vec![self.automobiles_start_address()],
        );
        return member.read().unwrap().to_vec();
    }

    pub fn read_entries(&self) -> Vec<Entry> {
        let member = DataMember::<[Entry; 6]>::new_offset(
            self.ps2.pcsx2_process_handle,
            vec![self.entries_start_address()],
        );
        return member.read().unwrap().to_vec();
    }
}
