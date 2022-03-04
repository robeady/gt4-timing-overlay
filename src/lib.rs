use game_data::GameData;
use hudhook::{apply_hook, cleanup_hooks, RenderContext, RenderLoop};
use log::{LevelFilter, Log, Metadata, Record};
use ps2_types::Ps2Memory;
use simplelog::{
    ColorChoice, CombinedLogger, Config, ConfigBuilder, SharedLogger, TermLogger, TerminalMode,
    ThreadLogMode, WriteLogger,
};
use std::{
    fs::File,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr::null_mut,
    thread,
};
use winapi::um::{
    consoleapi::AllocConsole,
    winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
};

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
        // I have no clue if this really is unwind safe, but this function is called by native code, and exposing it to rust panics cannot possibly be better
        if let Err(e) = catch_unwind(AssertUnwindSafe(|| {
            let scale = ctx.display_size[1] / 480.0;
            ui::render_ui(ctx.frame, [450., 300.], &mut self.game_data, true, scale)
        })) {
            log::error!("{:?}", e);
        }
    }

    fn is_visible(&self) -> bool {
        true
    }
    fn is_capturing(&self) -> bool {
        true
    }

    fn init(&mut self, imgui_context: &mut imgui::Context) {
        ui::init_ui(imgui_context, 1.0);
    }
}

// hudhook!(Box::new(MyRenderLoop));
// // expand it myself because of undeclared dependencies to do with logging...

#[no_mangle]
pub extern "stdcall" fn DllMain(
    _: winapi::shared::minwindef::HINSTANCE,
    reason: u32,
    lp_reserved: *mut winapi::ctypes::c_void,
) {
    match reason {
        DLL_PROCESS_ATTACH => {
            unsafe {
                AllocConsole();
            }
            if let Err(e) = thread::Builder::new().spawn(|| {
                let log_config = ConfigBuilder::new()
                    .set_thread_level(LevelFilter::Error)
                    .set_thread_mode(ThreadLogMode::Both)
                    .build();
                CombinedLogger::init(vec![
                    TermLogger::new(
                        LevelFilter::Debug,
                        log_config.clone(),
                        TerminalMode::Mixed,
                        ColorChoice::Auto,
                    ),
                    Box::new(AlwaysFlush(WriteLogger::new(
                        LevelFilter::Debug,
                        log_config.clone(),
                        File::create("gt4timing.log").unwrap(),
                    ))),
                ])
                .unwrap_or_else(|e| println!("{}", e));

                log::info!("Started thread, enabling hook");
                match apply_hook(Box::new(Gt4TimingRenderLoop {
                    game_data: GameData::in_same_process(),
                })) {
                    Ok(_) => log::info!("Hook enabled"),
                    Err(e) => log::error!("Hook errored: {:?}", e),
                }
            }) {
                println!("Error spawning thread: {}", e)
            }
        }
        DLL_PROCESS_DETACH if lp_reserved == null_mut() => {
            // lp_reserved == NULL => FreeLibrary called or DLL load failed, rather than process termination
            cleanup_hooks().unwrap_or_else(|e| log::error!("{}", e));
        }
        _ => {}
    }
}

struct AlwaysFlush(Box<dyn SharedLogger>);

impl Log for AlwaysFlush {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.0.enabled(metadata)
    }

    fn log(&self, record: &Record<'_>) {
        self.0.log(record);
        // here's the important bit:
        self.flush();
    }

    fn flush(&self) {
        self.0.flush()
    }
}

impl SharedLogger for AlwaysFlush {
    fn level(&self) -> LevelFilter {
        self.0.level()
    }

    fn config(&self) -> Option<&Config> {
        self.0.config()
    }

    fn as_log(self: Box<Self>) -> Box<dyn Log> {
        self.0.as_log()
    }
}
