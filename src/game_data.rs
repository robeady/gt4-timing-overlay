use std::{
    cmp::{min, Ordering},
    collections::BTreeMap,
    f32::INFINITY,
    mem::size_of,
};

use crate::{
    ps2_types::{Ps2InProcess, Ps2Memory, Ps2Ptr, Ps2PtrChain, Ps2SeparateProcess, Ps2String},
    scan_memory,
};
use derivative::Derivative;
use ordered_float::OrderedFloat;
use process_memory::{DataMember, Memory, ProcessHandle};

const NUM_CARS: usize = 6;

const BEFORE_NANS: usize = 140;

// 4256 bytes
// each struct starts a bit before the NaN block, guess 50 bytes
#[repr(C)]
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
    pub implicit_current_lap: i16, // from decompiler
    #[derivative(Debug = "ignore")]
    _unk100_2: [u8; 78],
    pub almost_rpm: f32, // + 1396
    #[derivative(Debug = "ignore")]
    _unk32: [u8; 32],
    pub gear: u8, // + 1432
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

impl Automobile {
    pub fn progress(&self, track_length: f32) -> OrderedFloat<f32> {
        let lap: f32 = self.implicit_current_lap.into();
        (lap + self.meters_driven_in_current_lap / track_length).into()
    }
}

// size unknown, at least 5500 bytes or so
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CarSpec {
    unknown: [u8; 72],
    pub mass: f32,
    // more stuff
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

type TimeMs = i32;

pub struct GameData<M: Ps2Memory> {
    pub ps2: M,
    /// for each car, a map from distance through the race to time at which this distance was reached
    pub car_checkpoints: [BTreeMap<OrderedFloat<f32>, TimeMs>; NUM_CARS],
    pub race_time: TimeMs,
}

// offset from EE main memory base to start of NaN block for cars[1]
const FIRST_NAN_OFFSET_FROM_EE_BASE: usize = 0x01C0EEA4;

impl GameData<Ps2InProcess> {
    pub fn in_same_process() -> Self {
        return GameData {
            ps2: Ps2InProcess,
            car_checkpoints: [
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
            ],
            race_time: 0,
        };
    }
}

impl GameData<Ps2SeparateProcess> {
    pub fn connect(process_handle: ProcessHandle) -> Self {
        let ee_base_address = 0x2000_0000;
        return GameData {
            ps2: Ps2SeparateProcess {
                ee_base_address,
                pcsx2_process_handle: process_handle,
            },
            car_checkpoints: [
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
            ],
            race_time: 0,
        };
    }
}

impl<M: Ps2Memory> GameData<M> {
    pub fn sample_car_checkpoints(&mut self) {
        let autos = self.read_cars();
        let new_race_time = self.read_race_time();
        if new_race_time < self.race_time {
            for i in 0..NUM_CARS {
                self.car_checkpoints[i].clear()
            }
        }
        self.race_time = new_race_time;
        let track_length = self.read_track_length();
        for i in 0..NUM_CARS {
            let progress = autos[i].progress(track_length);
            if progress >= 1f32.into() {
                self.car_checkpoints[i].insert(progress, self.race_time);
            }
        }
    }

    pub fn gap_to_leader_ms(&self, car: usize) -> Option<f32> {
        let cars = self.read_cars();
        let current_time = self.read_race_time() as f32;
        let track_length = self.read_track_length();
        let progress_to_find = cars[car].progress(track_length);
        if progress_to_find < 1f32.into() {
            // cars still on the grid
            return None;
        }
        let mut leader_time: Option<f32> = None;
        for i in 0..NUM_CARS {
            if i == car {
                continue;
            }
            let min_greater = self.car_checkpoints[i].range(progress_to_find..).next();
            let max_less = self.car_checkpoints[i]
                .range(..progress_to_find)
                .next_back();

            if let (Some(min_greater), Some(max_less)) = (min_greater, max_less) {
                // linearly interpolate
                let alpha: OrderedFloat<f32> =
                    (progress_to_find - *max_less.0) / (*min_greater.0 - *max_less.0);
                let interpolated_time: f32 =
                    *max_less.1 as f32 + alpha.into_inner() * (*min_greater.1 - *max_less.1) as f32;

                if let Some(t) = leader_time {
                    if t > interpolated_time {
                        leader_time = Some(interpolated_time)
                    }
                } else {
                    leader_time = Some(interpolated_time);
                }
            }
        }
        return leader_time.map(|t| current_time - t);
    }

    pub fn read_cars(&self) -> Vec<Automobile> {
        let cars_start_address = FIRST_NAN_OFFSET_FROM_EE_BASE
        - BEFORE_NANS  // go to start of Automobile struct
        - size_of::<Automobile>(); // that was entry 1, go to entry 0
        Ps2Ptr::<[Automobile; 6]>::new(cars_start_address as u32)
            .get(&self.ps2)
            .to_vec()
    }

    pub fn read_entries(&self) -> Vec<Entry> {
        let entries_start_address = FIRST_NAN_OFFSET_FROM_EE_BASE - 0x2E0A4;
        Ps2Ptr::<[Entry; 6]>::new(entries_start_address as u32)
            .get(&self.ps2)
            .to_vec()
    }

    pub fn read_race_time(&self) -> TimeMs {
        let race_time_address = FIRST_NAN_OFFSET_FROM_EE_BASE - 0xA4A0;
        Ps2Ptr::new(race_time_address as u32).get(&self.ps2)
    }

    pub fn read_track_length(&self) -> f32 {
        Ps2PtrChain::new(&[0x01BF52FC, 404, 20]).get(&self.ps2)
    }
}
