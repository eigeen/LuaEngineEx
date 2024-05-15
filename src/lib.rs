#![allow(non_snake_case)]

use std::collections::HashMap;
use std::f32::consts::E;
use std::sync::{Arc, Once};
use std::thread;

use log::{debug, error, info};
use luavm::{LuaHandler, LuaVMError};
use mhw_toolkit::game::hooks::{CallbackPosition, HookHandle};
use snafu::prelude::*;
use tokio::runtime::Handle;
use tokio::sync::{mpsc, Mutex};
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
    #[snafu(display("Hook error: {}", source))]
    Hook { source: hooks::HookError },
    #[snafu(display("IO error: {}", source))]
    Io { source: std::io::Error },
    #[snafu(display("Error: {}", reason))]
    User { reason: String },
}

struct LuaManager {
    vm: HashMap<String, LuaHandler>,
}

impl LuaManager {
    pub fn new() -> Self {
        Self { vm: HashMap::new() }
    }

    pub async fn load_all(&mut self) -> Result<()> {
        for entry in std::fs::read_dir("LuaEngineEx").context(IoSnafu)? {
            let entry = entry.context(IoSnafu)?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "lua" {
                        debug!("loading lua file: {}", path.display());
                        let mut vm = LuaHandler::new(path.file_name().unwrap().to_str().unwrap());
                        vm.load_file(&path).await.context(LuaVMSnafu)?;
                        let name = vm.data.lock().await.name.clone();
                        debug!("Lua VM {} loaded", name);
                        self.vm.insert(name, vm);
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

    pub async fn run_all(&self) -> Result<()> {
        for vm in self.vm.values() {
            vm.run().await.context(LuaVMSnafu)?;
        }

        Ok(())
    }

    pub fn unload(&mut self, name: &str) -> Result<()> {
        // TODO: stop before unload
        self.vm.remove(name);

        Ok(())
    }

    pub async fn reload(&mut self, name: &str) -> Result<()> {
        // find in scheduler
        if self.vm.contains_key(name) {
            let vm = self.vm.get_mut(name).unwrap();
            match vm.reload().await {
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
                let mut vm = LuaHandler::new(path.file_name().unwrap().to_str().unwrap());
                vm.load_file(&path).await.context(LuaVMSnafu)?;
                let name = vm.data.lock().await.name.clone();
                self.vm.insert(name, vm);
                break;
            }
        }

        Err(Error::User {
            reason: "script not found".to_string(),
        })
    }

    pub async fn reload_all(&mut self) -> Result<()> {
        self.unload_all();
        self.load_all().await?;
        self.run_all().await?;

        Ok(())
    }
}

#[derive(Clone)]
pub enum ManagerEvent {
    ReloadAll,
    Reload(String),
}

async fn lua_main() -> Result<(), Error> {
    let (manager_tx, mut manager_rx) = mpsc::channel(128);
    // in game chat command listener
    let tx1 = manager_tx.clone();
    let mut hook_input_dispatch = mhw_toolkit::game::hooks::InputDispatchHook::new();
    hook_input_dispatch
        .set_hook(CallbackPosition::Before, move |input| {
            if !input.starts_with("/lua ") {
                return;
            }

            let inputs = input.split_whitespace().collect::<Vec<&str>>();
            if inputs.len() < 2 {
                return;
            }
            debug!("user command: {:?}", inputs);
            if inputs[1] == "debug" {
                // debug!("vm: {:#?}", vm_manager1.lock().unwrap().vm);
            } else if inputs[1] == "reload" {
                if inputs.len() < 3 {
                    // reload all
                    if let Err(e) = tx1.blocking_send(ManagerEvent::ReloadAll) {
                        error!("reload all error: {}", e);
                    };
                } else {
                    // reload specified
                    if let Err(e) = tx1.blocking_send(ManagerEvent::Reload(inputs[2].to_string())) {
                        error!("reload `{}` error: {}", inputs[2], e)
                    };
                }
            }
        })
        .map_err(|e| hooks::HookError::Hook {
            source: e,
            reason: "Failed to set input dispatch hook".to_string(),
        })
        .context(HookSnafu)?;

    // init basic services
    hooks::monster::init_monster_hooks().context(HookSnafu)?;

    // start lua main thread
    let vm_manager = Arc::new(Mutex::new(LuaManager::new()));
    // let handle = Handle::current();
    // thread::spawn(move || {
    //     handle.block_on(async {
    //         vm_manager.lock().await.load_all()?;
    //         vm_manager.lock().await.run_all().await?;
    //         while let Some(event) = manager_rx.recv().await {
    //             match event {
    //                 ManagerEvent::ReloadAll => {
    //                     if let Err(e) = vm_manager.lock().await.reload_all().await {
    //                         error!("reload error: {}", e);
    //                     } else {
    //                         info!("reload all successfully");
    //                     }
    //                 }
    //                 ManagerEvent::Reload(name) => {
    //                     if let Err(e) = vm_manager.lock().await.reload(&name).await {
    //                         error!("reload error: {}", e);
    //                     } else {
    //                         info!("reload {} successfully", name);
    //                     }
    //                 }
    //             }
    //         }
    //     });
    // });
    if let Err(e) = vm_manager.lock().await.load_all().await {
        error!("load error: {}", e);
    };
    if let Err(e) = vm_manager.lock().await.run_all().await {
        error!("run error: {}", e);
    };

    debug!("start command recv");
    loop {
        if let Some(event) = manager_rx.recv().await {
            match event {
                ManagerEvent::ReloadAll => {
                    error!("reload all");
                    if let Err(e) = vm_manager.lock().await.reload_all().await {
                        error!("reload error: {}", e);
                    } else {
                        info!("reload all successfully");
                    }
                }
                ManagerEvent::Reload(name) => {
                    if let Err(e) = vm_manager.lock().await.reload(&name).await {
                        error!("reload error: {}", e);
                    } else {
                        info!("reload {} successfully", name);
                    }
                }
            }
        } else {
            error!("Command handler channel closed");
            break;
        }
    }

    // start module services
    // tokio::spawn(async move {});

    Ok(())
}

fn main_entry() -> Result<(), Error> {
    init_log();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context(IoSnafu)?;
    runtime.block_on(async {
        let _ = lua_main().await;
    });

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
