use std::marker::PhantomData;

use process_memory::Memory;
use process_memory::{DataMember, ProcessHandle};

pub struct Ps2Memory {
    pub pcsx2_process_handle: ProcessHandle,
    pub ee_base_address: usize,
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Ps2Ptr<T>(u32, PhantomData<T>);

impl<T: Copy> Ps2Ptr<T> {
    pub fn new(offset: u32) -> Self {
        Self(offset, PhantomData)
    }

    pub fn get(&self, ps2_memory: &Ps2Memory) -> T {
        let ptr = self.0 as usize;
        let addr = match ptr {
            0x00000000..=0x01FFFFFF => ps2_memory.ee_base_address + ptr,
            0x20000000..=0x21FFFFFF => ps2_memory.ee_base_address + ptr - 0x20000000,
            0x30000000..=0x31FFFFFF => ps2_memory.ee_base_address + ptr - 0x30000000,
            _ => panic!("unsupported PS2 pointer address {}", ptr),
        };
        return DataMember::new_offset(ps2_memory.pcsx2_process_handle, vec![addr])
            .read()
            .unwrap();
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

    pub fn get(&self, ps2_memory: &Ps2Memory) -> T {
        let mut ptr = 0u32;
        let (&last_offset, offsets) = self.0.split_last().unwrap();
        for &offset in offsets {
            let addr = ptr + offset;
            let mapped_addr = match addr {
                0x00000000..=0x01FFFFFF => ps2_memory.ee_base_address as u32 + addr,
                0x20000000..=0x21FFFFFF => ps2_memory.ee_base_address as u32 + addr - 0x20000000,
                0x30000000..=0x31FFFFFF => ps2_memory.ee_base_address as u32 + addr - 0x30000000,
                _ => panic!("unsupported PS2 pointer address {:x} a", addr),
            } as usize;
            ptr = DataMember::<u32>::new_offset(ps2_memory.pcsx2_process_handle, vec![mapped_addr])
                .read()
                .unwrap();
        }
        let addr = ptr + last_offset;
        let mapped_addr = match addr {
            0x00000000..=0x01FFFFFF => ps2_memory.ee_base_address as u32 + addr,
            0x20000000..=0x21FFFFFF => ps2_memory.ee_base_address as u32 + addr - 0x20000000,
            0x30000000..=0x31FFFFFF => ps2_memory.ee_base_address as u32 + addr - 0x30000000,
            _ => panic!("unsupported PS2 pointer address {:x} a", addr),
        } as usize;
        DataMember::<T>::new_offset(ps2_memory.pcsx2_process_handle, vec![mapped_addr])
            .read()
            .unwrap()
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
