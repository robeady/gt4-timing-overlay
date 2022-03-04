use std::marker::PhantomData;

use anyhow::{bail, Result};
use process_memory::{DataMember, ProcessHandle};
use process_memory::{LocalMember, Memory};

const EE_BASE_ADDRESS: u32 = 0x20000000;

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Ps2Ptr<T>(u32, PhantomData<T>);

impl<T> Ps2Ptr<T> {
    pub const fn new(offset: u32) -> Self {
        Self(offset, PhantomData)
    }
}

impl<T: Copy> Ps2Ptr<T> {
    pub fn get<M: Ps2Memory>(&self, ps2_memory: &M) -> Result<T> {
        ps2_memory.read(self.0)
    }
}

pub struct Ps2PtrChain<T>(Vec<u32>, PhantomData<T>);

impl<T> Ps2PtrChain<T> {
    pub const fn new(offsets: Vec<u32>) -> Self {
        Self(offsets, PhantomData)
    }
}

impl<T: Copy> Ps2PtrChain<T> {
    pub fn get<M: Ps2Memory>(&self, ps2_memory: &M) -> Result<T> {
        let mut ptr = 0u32;
        let (&last_offset, offsets) = self.0.split_last().expect("pointer chain has no offsets");
        for (step, &offset) in offsets.iter().enumerate() {
            let addr = ptr + offset;
            ptr = ps2_memory.read::<u32>(addr)?;
            if ptr == 0 {
                bail!("null pointer found at {:x} in chain {:?}[{}]", addr, self.0, step);
            }
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
        return s.0.iter().take_while(|&c| *c != 0).map(|&c| c as char).collect();
    }
}

pub trait Ps2Memory {
    fn read<T: Copy>(&self, address: u32) -> Result<T>;
}

pub struct Ps2SeparateProcess {
    pub pcsx2_process_handle: ProcessHandle,
}

impl Ps2Memory for Ps2SeparateProcess {
    fn read<T: Copy>(&self, address: u32) -> Result<T> {
        let mapped_addr = remap_ps2_address(address);
        Ok(DataMember::new_offset(self.pcsx2_process_handle, vec![mapped_addr as usize]).read()?)
    }
}

fn remap_ps2_address(address: u32) -> u32 {
    match address {
        0x00000000..=0x01FFFFFF => EE_BASE_ADDRESS + address,
        0x20000000..=0x21FFFFFF => EE_BASE_ADDRESS + address - 0x20000000,
        0x30000000..=0x31FFFFFF => EE_BASE_ADDRESS + address - 0x30000000,
        _ => panic!("unsupported PS2 pointer address {:x}", address),
    }
}

pub struct Ps2InProcess;

impl Ps2Memory for Ps2InProcess {
    fn read<T: Copy>(&self, address: u32) -> Result<T> {
        let mapped_addr = remap_ps2_address(address);
        Ok(LocalMember::new_offset(vec![mapped_addr as usize]).read()?)
    }
}
