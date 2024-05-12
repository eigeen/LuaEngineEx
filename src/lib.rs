#![allow(non_snake_case)]

use std::sync::Once;
use std::thread;

use log::{debug, error};
use luavm::{LuaVM, LuaVMError};
use snafu::prelude::*;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE};
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

static MAIN_THREAD_ONCE: Once = Once::new();

mod hooks;
mod logger;
mod luavm;

mod use_logger {
    use log::LevelFilter;
    use once_cell::sync::Lazy;

    use crate::logger::MHWLogger;

    static LOGGER: Lazy<MHWLogger> = Lazy::new(MHWLogger::new);

    pub fn init_log() {
        log::set_logger(&*LOGGER).unwrap();
        log::set_max_level(LevelFilter::Debug);
    }
}

use use_logger::init_log;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Lua error: {}", source))]
    LuaVM { source: LuaVMError },
    #[snafu(display("Hook error: {}", reason))]
    Hook { reason: String },
    #[snafu(display("IO error: {}", source))]
    Io { source: std::io::Error },
}

struct LuaVMManager {
    vm: Vec<LuaVM>,
}

impl LuaVMManager {
    pub fn new() -> Self {
        Self { vm: Vec::new() }
    }

    pub fn load_all(&mut self) -> Result<()> {
        for entry in std::fs::read_dir("LuaEngineEx").context(IoSnafu)? {
            let entry = entry.context(IoSnafu)?;
            let path = entry.path();
            if path.is_file() {
                debug!("loading lua file: {}", path.display());
                let mut vm = LuaVM::new(path.file_name().unwrap().to_str().unwrap());
                vm.load_file(&path).context(LuaVMSnafu)?;
                self.vm.push(vm);
            }
        }

        Ok(())
    }

    pub fn unload_all(&mut self) {
        // TODO: stop before unload
        self.vm.clear();
    }

    pub fn run_all(&self) -> Result<()> {
        for vm in &self.vm {
            vm.run().context(LuaVMSnafu)?;
        }

        Ok(())
    }

    pub fn unload(&mut self, name: &str) -> Result<()> {
        for vm in &self.vm {
            vm.run().context(LuaVMSnafu)?;
        }
        // TODO: stop before unload
        self.vm.retain(|vm| vm.data.name != name);

        Ok(())
    }
}

fn lua_main() -> Result<(), Error> {
    let mut vm_manager = LuaVMManager::new();
    vm_manager.load_all()?;
    vm_manager.run_all()?;

    // let mut test_vm = LuaVM::new();
    // test_vm
    //     .load_file("LuaEngineEx/test.lua")
    //     .context(LuaVMSnafu)?;
    // test_vm.run().context(LuaVMSnafu)?;

    Ok(())
}

fn main_entry() -> Result<(), Error> {
    init_log();
    // in game chat command listener
    mhw_toolkit::game::hooks::hook_input_dispatch(|input| {
        // todo!
        if input.starts_with("/lua") {
            debug!("user command: {}", input);
            // TODO
        }
    })
    .map_err(|e| Error::Hook {
        reason: e.to_string(),
    })?;

    lua_main()?;

    Ok(())
}

#[no_mangle]
#[allow(unused_variables)]
extern "stdcall" fn DllMain(dll_module: HINSTANCE, call_reason: DWORD, reserved: LPVOID) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            MAIN_THREAD_ONCE.call_once(|| {
                thread::spawn(|| {
                    if let Err(e) = main_entry() {
                        error!("runtime error: {}", e);
                    }
                });
            });
        }
        DLL_PROCESS_DETACH => (),
        _ => (),
    }
    TRUE
}
