use std::ffi::CString;

use hudhook::Error;
use winapi::shared::minwindef::*;
use winapi::um::handleapi::*;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::um::memoryapi;
use winapi::um::minwinbase::LPSECURITY_ATTRIBUTES;
use winapi::um::processthreadsapi;
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::INFINITE;
use winapi::um::winnt::*;

/// basically copied from hudhook, but with extra code to grab the HMODULE of the created module
pub fn inject(pid: u32, dll_path: &str) -> Result<u32, Error> {
    let pathstr = std::fs::canonicalize(dll_path).map_err(|e| Error::from(format!("{:?}", e)))?;
    let mut path = [0i8; MAX_PATH];
    for (dest, src) in path.iter_mut().zip(
        CString::new(pathstr.to_str().unwrap())
            .unwrap()
            .into_bytes()
            .into_iter(),
    ) {
        *dest = src as _;
    }

    let hproc = unsafe { processthreadsapi::OpenProcess(PROCESS_ALL_ACCESS, 0, pid) };
    let dllp = unsafe {
        memoryapi::VirtualAllocEx(
            hproc,
            0 as LPVOID,
            MAX_PATH,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_READWRITE,
        )
    };

    unsafe {
        memoryapi::WriteProcessMemory(
            hproc,
            dllp,
            std::mem::transmute(&path),
            MAX_PATH,
            std::ptr::null_mut::<usize>(),
        );
    }

    let thread = unsafe {
        let kernel32 = CString::new("Kernel32").unwrap();
        let loadlibrarya = CString::new("LoadLibraryA").unwrap();
        let proc_addr = GetProcAddress(GetModuleHandleA(kernel32.as_ptr()), loadlibrarya.as_ptr());
        processthreadsapi::CreateRemoteThread(
            hproc,
            0 as LPSECURITY_ATTRIBUTES,
            0,
            Some(std::mem::transmute(proc_addr)),
            dllp,
            0,
            std::ptr::null_mut::<DWORD>(),
        )
    };
    // println!("{:?}", thread);

    let mut hmodule = 0u32;
    unsafe {
        WaitForSingleObject(thread, INFINITE);
        // CAREFUL: this only works on 32-bit where HMODULE is a DWORD
        processthreadsapi::GetExitCodeThread(thread, &mut hmodule as *mut DWORD);
        CloseHandle(thread);
        memoryapi::VirtualFreeEx(hproc, dllp, 0, MEM_RELEASE);
        CloseHandle(hproc);
    };

    Ok(hmodule)
}

/// this function uninjects a DLL that was injected earlier by calling FreeLibraryA.
pub fn uninject(pid: u32, module_handle: u32) -> Result<u32, Error> {
    let hproc = unsafe { processthreadsapi::OpenProcess(PROCESS_ALL_ACCESS, 0, pid) };

    let thread = unsafe {
        let kernel32 = CString::new("Kernel32").unwrap();
        let free_library_a = CString::new("FreeLibrary").unwrap();
        let proc_addr =
            GetProcAddress(GetModuleHandleA(kernel32.as_ptr()), free_library_a.as_ptr());
        processthreadsapi::CreateRemoteThread(
            hproc,
            0 as LPSECURITY_ATTRIBUTES,
            0,
            Some(std::mem::transmute(proc_addr)),
            module_handle as _,
            0,
            std::ptr::null_mut::<DWORD>(),
        )
    };
    // println!("{:?}", thread);

    let mut ec = 0u32;
    unsafe {
        WaitForSingleObject(thread, INFINITE);

        processthreadsapi::GetExitCodeThread(thread, &mut ec as *mut DWORD);
        CloseHandle(thread);
        CloseHandle(hproc);
    };

    Ok(ec)
}
