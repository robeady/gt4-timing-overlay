use game_data::GameData;
use hudhook::{apply_hook, cleanup_hooks, RenderContext, RenderLoop};
use ps2_types::Ps2Memory;
use std::{ptr::null_mut, thread};
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

mod game_data;
mod processes;
mod ps2_types;
mod scan_memory;
mod ui;
mod window;

pub struct Gt4TimingRenderLoop<M: Ps2Memory> {
    game_data: GameData<M>,
}

impl<M: Ps2Memory> RenderLoop for Gt4TimingRenderLoop<M> {
    fn render(&mut self, ctx: RenderContext) {
        ui::render_ui(ctx.frame, [320., 300.], &mut self.game_data, true)
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
                match apply_hook(Box::new(Gt4TimingRenderLoop { game_data: GameData::in_same_process() })) {
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
