// use hudhook::*;
use hudhook::imgui::{im_str, Condition, Window};
use hudhook::{apply_hook, cleanup_hooks, RenderContext, RenderLoop};
use std::{ptr::null_mut, thread};
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

pub struct MyRenderLoop;

impl RenderLoop for MyRenderLoop {
    fn render(&mut self, ctx: RenderContext) {
        Window::new(im_str!("My first render loop"))
            .size([320., 200.], Condition::FirstUseEver)
            .position([0., 0.], Condition::FirstUseEver)
            .build(ctx.frame, || {
                ctx.frame.text(im_str!("Hello, hello!"));
            });
    }

    fn is_visible(&self) -> bool {
        true
    }
    fn is_capturing(&self) -> bool {
        true
    }
}

// hudhook!(Box::new(MyRenderLoop));
// // expand it myself because of undeclared dependencies to do with logging...

/// Entry point created by the `hudhook` library.
#[no_mangle]
pub extern "stdcall" fn DllMain(
    _: winapi::shared::minwindef::HINSTANCE,
    reason: u32,
    lp_reserved: *mut winapi::ctypes::c_void,
) {
    match reason {
        DLL_PROCESS_ATTACH => {
            thread::spawn(|| {
                println!("Started thread, enabling hook...");
                match apply_hook(Box::new(MyRenderLoop)) {
                    Ok(_) => println!("Hook enabled"),
                    Err(e) => println!("Hook errored: {:?}", e),
                }
            });
        }
        DLL_PROCESS_DETACH if lp_reserved == null_mut() /* NULL => FreeLibrary called or DLL load failed, rather than process termination */ => {
            cleanup_hooks().unwrap_or_else(|e| println!("{}", e));
        }
        _ => {}
    }
}
