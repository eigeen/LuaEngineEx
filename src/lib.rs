#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::{Arc, Once};
use std::thread;

use clap::Parser;
use log::{debug, error, info};
use luavm::{LuaVM, LuaVMError};
use snafu::prelude::*;
use std::sync::Mutex as StdMutex;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE};
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

static MAIN_THREAD_ONCE: Once = Once::new();

mod command;
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
    #[snafu(display("Error: {}", reason))]
    User { reason: String },
}

struct LuaVMManager {
    vm: HashMap<String, LuaVM>,
}

impl LuaVMManager {
    pub fn new() -> Self {
        Self { vm: HashMap::new() }
    }

    pub fn load_all(&mut self) -> Result<()> {
        for entry in std::fs::read_dir("LuaEngineEx").context(IoSnafu)? {
            let entry = entry.context(IoSnafu)?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "lua" {
                        debug!("loading lua file: {}", path.display());
                        let mut vm = LuaVM::new(path.file_name().unwrap().to_str().unwrap());
                        vm.load_file(&path).context(LuaVMSnafu)?;
                        debug!("Lua VM {} loaded", vm.data.name);
                        self.vm.insert(vm.data.name.clone(), vm);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn unload_all(&mut self) {
        // TODO: stop before unload
        self.vm.clear();
    }

    pub fn run_all(&self) -> Result<()> {
        for vm in self.vm.values() {
            vm.run().context(LuaVMSnafu)?;
        }

        Ok(())
    }

    pub fn unload(&mut self, name: &str) -> Result<()> {
        // TODO: stop before unload
        self.vm.remove(name);

        Ok(())
    }

    pub fn reload(&mut self, name: &str) -> Result<()> {
        // find in scheduler
        if self.vm.contains_key(name) {
            let vm = self.vm.get_mut(name).unwrap();
            match vm.reload() {
                Ok(_) => return Ok(()),
                Err(e) => {
                    return Err(Error::LuaVM { source: e });
                }
            };
        }
        // find in fs
        for entry in std::fs::read_dir("LuaEngineEx").context(IoSnafu)? {
            let entry = entry.context(IoSnafu)?;
            let path = entry.path();
            if path.is_file() && path.file_name().unwrap().to_str().unwrap() == name {
                debug!("loading lua file: {}", path.display());
                let mut vm = LuaVM::new(path.file_name().unwrap().to_str().unwrap());
                vm.load_file(&path).context(LuaVMSnafu)?;
                self.vm.insert(vm.data.name.clone(), vm);
                break;
            }
        }

        Err(Error::User {
            reason: "script not found".to_string(),
        })
    }

    pub fn reload_all(&mut self) -> Result<()> {
        self.unload_all();
        self.load_all()?;
        self.run_all()?;

        Ok(())
    }
}

async fn lua_main() -> Result<(), Error> {
    let vm_manager = Arc::new(StdMutex::new(LuaVMManager::new()));

    // in game chat command listener
    let vm_manager1 = vm_manager.clone();
    mhw_toolkit::game::hooks::hook_input_dispatch(move |input| {
        if input.starts_with("/lua ") {
            let inputs = input.split_whitespace().collect::<Vec<&str>>();
            if inputs.len() < 2 {
                return;
            }
            debug!("user command: {:?}", inputs);
            if inputs[1] == "debug" {
                debug!("vm: {:#?}", vm_manager1.lock().unwrap().vm);
            } else if inputs[1] == "reload" {
                if inputs.len() < 3 {
                    // reload all
                    if let Err(e) = vm_manager1.lock().unwrap().reload_all() {
                        error!("reload error: {}", e);
                    } else {
                        info!("reload all successfully");
                    }
                } else {
                    // reload specified
                    if let Err(e) = vm_manager1.lock().unwrap().reload(inputs[2]) {
                        error!("reload error: {}", e);
                    } else {
                        info!("reload {} successfully", inputs[2]);
                    }
                }
            }
            // match command::Cli::try_parse_from(inputs) {
            //     Ok(cli) => match cli.command {
            //         command::Command::Reload { script } => match script {
            //             Some(script) => {
            //                 if let Err(e) = vm_manager1.lock().unwrap().reload(&script) {
            //                     error!("reload error: {}", e);
            //                 }
            //             }
            //             None => {
            //                 if let Err(e) = vm_manager1.lock().unwrap().reload_all() {
            //                     error!("reload error: {}", e);
            //                 }
            //             }
            //         },
            //         command::Command::Debug { command } => {
            //             match command {
            //                 command::DebugCommand::Vm => {
            //                     debug!("vm: {:#?}", vm_manager1.lock().unwrap().vm);
            //                 },
            //             }
            //         },
            //     },
            //     Err(e) => {
            //         error!("{}", e);
            //     }
            // }
        }
    })
    .map_err(|e| Error::Hook {
        reason: e.to_string(),
    })?;

    vm_manager.lock().unwrap().load_all()?;
    vm_manager.lock().unwrap().run_all()?;

    Ok(())
}

fn main_entry() -> Result<(), Error> {
    init_log();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context(IoSnafu)?;
    runtime.block_on(lua_main())?;

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
