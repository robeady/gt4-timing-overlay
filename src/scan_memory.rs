use std::mem::size_of;

use process_memory::{DataMember, Memory, ProcessHandle};
use winapi::{
    shared::minwindef::{HMODULE, LPCVOID},
    um::{
        memoryapi::VirtualQueryEx,
        psapi::GetModuleFileNameExW,
        winnt::{MEMORY_BASIC_INFORMATION, PAGE_READWRITE, PMEMORY_BASIC_INFORMATION},
    },
};

/// Finds all occurrences of needle in the private RW memory of the given process.
pub fn find_all_offsets(needle: &[u8], handle: ProcessHandle) -> Vec<usize> {
    const CHUNK_SIZE: usize = 4096;
    if needle.len() > CHUNK_SIZE {
        panic!("needle too long")
    }

    // first, figure out which bits of memory are safe to scan
    let mut regions_of_interest = Vec::new();
    let mut mbi: MEMORY_BASIC_INFORMATION = Default::default();
    let mut address = 0 as LPCVOID;
    loop {
        let result = unsafe {
            VirtualQueryEx(
                handle.0,
                address,
                &mut mbi as PMEMORY_BASIC_INFORMATION,
                size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };
        if result == 0 {
            break;
        } else {
            // find out what module it is
            let mut v = [0u16; 255];
            let len = unsafe {
                GetModuleFileNameExW(
                    handle.0,
                    mbi.AllocationBase as HMODULE,
                    v.as_mut_ptr(),
                    v.len() as u32,
                ) as usize
            };
            // only read-write pages with no module name are of interest to us (if the game can't write it, it's not gonna be interesting)
            if len == 0 && mbi.Protect == PAGE_READWRITE {
                regions_of_interest.push(mbi.clone())
            }
            // advance to next region
            address = ((mbi.BaseAddress as usize) + mbi.RegionSize) as LPCVOID;
        }
    }

    let mut matches = Vec::new();

    for region in regions_of_interest.iter() {
        // println!("scanning {:?}", DebugMBI(region));
        let lower = region.BaseAddress as usize;
        let upper = lower + region.RegionSize;
        let mut chunk_offset = lower;
        while chunk_offset < upper {
            let member = DataMember::<[u8; CHUNK_SIZE]>::new_offset(handle, vec![chunk_offset]);
            let haystack = member.read().unwrap();
            haystack
                .windows(needle.len())
                .enumerate()
                .filter_map(|(offset, window)| if window == needle { Some(offset) } else { None })
                .for_each(|offset| matches.push(chunk_offset + offset));

            // TODO: this chunk handling is rather hacky
            // need overlap between chunks
            let old_chunk_offset = chunk_offset;
            chunk_offset = chunk_offset + CHUNK_SIZE - needle.len();
            // small correction to avoid going out of bounds at the end
            if chunk_offset < upper && chunk_offset + CHUNK_SIZE >= upper {
                chunk_offset = upper - CHUNK_SIZE;
            }
            if old_chunk_offset == chunk_offset {
                break;
            }
        }
    }

    return matches;
}

// struct DebugMBI<'a>(&'a MEMORY_BASIC_INFORMATION);

// impl<'a> fmt::Debug for DebugMBI<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("MEMORY_BASIC_INFORMATION")
//             .field("BaseAddress", &self.0.BaseAddress)
//             .field("AllocationBase", &self.0.AllocationBase)
//             .field("AllocationProtect", &self.0.AllocationProtect)
//             .field("RegionSize", &self.0.RegionSize)
//             .field("State", &self.0.State)
//             .field("Protect", &self.0.Protect)
//             .field("Type", &self.0.Type)
//             .finish()
//     }
// }
