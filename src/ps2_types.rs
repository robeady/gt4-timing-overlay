use std::marker::PhantomData;

use process_memory::{DataMember, ProcessHandle};
use process_memory::{LocalMember, Memory};

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Ps2Ptr<T>(u32, PhantomData<T>);

impl<T: Copy> Ps2Ptr<T> {
    pub fn new(offset: u32) -> Self {
        Self(offset, PhantomData)
    }

    pub fn get<M: Ps2Memory>(&self, ps2_memory: &M) -> T {
        ps2_memory.read(self.0)
    }
}

pub struct Ps2PtrChain<'a, T>(&'a [u32], PhantomData<T>);

impl<'a, T: Copy> Ps2PtrChain<'a, T> {
    pub fn new(offsets: &'a [u32]) -> Self {
        if offsets.len() == 0 {
            panic!("0 offsets provided when creating pointer chain")
        }
        Self(offsets, PhantomData)
    }

    pub fn get<M: Ps2Memory>(&self, ps2_memory: &M) -> T {
        let mut ptr = 0u32;
        let (&last_offset, offsets) = self.0.split_last().unwrap();
        for &offset in offsets {
            ptr = ps2_memory.read::<u32>(ptr + offset);
        }
        ps2_memory.read(ptr + last_offset)
    }
}

/// A fixed length inline string, padded with zeros
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Ps2String<const N: usize>([u8; N]);

impl<const N: usize> From<Ps2String<N>> for String {
    fn from(s: Ps2String<N>) -> Self {
        return s
            .0
            .iter()
            .take_while(|&c| *c != 0)
            .map(|&c| c as char)
            .collect();
    }
}

pub trait Ps2Memory {
    fn read<T: Copy>(&self, address: u32) -> T;
}

pub struct Ps2SeparateProcess {
    pub pcsx2_process_handle: ProcessHandle,
    pub ee_base_address: usize,
}

impl Ps2Memory for Ps2SeparateProcess {
    fn read<T: Copy>(&self, address: u32) -> T {
        let mapped_addr = match address {
            0x00000000..=0x01FFFFFF => self.ee_base_address as u32 + address,
            0x20000000..=0x21FFFFFF => self.ee_base_address as u32 + address - 0x20000000,
            0x30000000..=0x31FFFFFF => self.ee_base_address as u32 + address - 0x30000000,
            _ => panic!("unsupported PS2 pointer address {:x} a", address),
        };
        DataMember::new_offset(self.pcsx2_process_handle, vec![mapped_addr as usize])
            .read()
            .unwrap()
    }
}

pub struct Ps2InProcess;

impl Ps2Memory for Ps2InProcess {
    fn read<T: Copy>(&self, address: u32) -> T {
        // TODO: check whether any memory mapping is required in-process or whether we can read the address directly
        // I don't know enough about how the emulator works...
        LocalMember::new_offset(vec![address as usize])
            .read()
            .unwrap()
    }
}
