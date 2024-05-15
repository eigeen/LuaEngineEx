use std::{
    path::Path,
    sync::{Arc, Weak},
};

use log::debug;
use mlua::prelude::*;
use snafu::prelude::*;
use tokio::sync::Mutex;

use super::libs;

pub type WeakLuaVM = Weak<Mutex<LuaVM>>;

#[derive(Debug, Snafu)]
pub enum LuaVMError {
    #[snafu(display("Failed to load script file: {}", source))]
    LoadFile { source: std::io::Error },
    #[snafu(display("Lua VM runs before loading script"))]
    NotLoaded,
    #[snafu(display("Failed to load script: {}", source))]
    LuaRuntime { source: mlua::Error },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RinningState {
    Unloaded,
    Loaded,
    Running,
}

#[derive(Debug)]
pub struct LuaVM {
    pub lua: Lua,
    running_state: RinningState,
}

impl LuaVM {
    pub fn new() -> Self {
        Self {
            lua: Lua::new(),
            running_state: RinningState::Unloaded,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running_state == RinningState::Running
    }

    pub async fn run(&mut self, script: &str) -> LuaResult<()> {
        self.lua.load(script).exec_async().await?;
        self.running_state = RinningState::Running;
        Ok(())
    }
}

#[derive(Debug)]
pub struct LuaHandlerData {
    pub name: String,
    pub file_path: Option<String>,
    pub script: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LuaHandler {
    pub data: Arc<Mutex<LuaHandlerData>>,
    luavm: Arc<Mutex<LuaVM>>,
}

impl LuaHandler {
    pub fn new(name: &str) -> Self {
        Self {
            data: Arc::new(Mutex::new(LuaHandlerData {
                name: name.to_string(),
                file_path: None,
                script: None,
            })),
            luavm: Arc::new(Mutex::new(LuaVM::new())),
        }
    }

    async fn run_inner(&self, script: &str) -> LuaResult<()> {
        self.load_libs().await?;
        self.luavm.lock().await.run(script).await?;

        Ok(())
    }

    async fn load_libs(&self) -> LuaResult<()> {
        libs::load_libs(self.get_luavm_weak()).await
    }

    pub fn get_luavm_weak(&self) -> WeakLuaVM {
        Arc::downgrade(&self.luavm)
    }

    pub async fn run(&self) -> Result<(), LuaVMError> {
        let data = self.data.lock().await;
        if data.script.is_none() {
            return Err(LuaVMError::NotLoaded);
        }

        let script = &data.script.clone().unwrap();
        debug!("Lua VM `{}` start running", data.name);
        self.run_inner(script)
            .await
            .map_err(|e| LuaVMError::LuaRuntime { source: e })?;

        Ok(())
    }

    pub async fn load_file<P>(&mut self, file_path: P) -> Result<(), LuaVMError>
    where
        P: AsRef<Path>,
    {
        let script = std::fs::read_to_string(&file_path).context(LoadFileSnafu)?;
        let mut data = self.data.lock().await;
        data.file_path = Some(file_path.as_ref().to_string_lossy().to_string());
        data.script = Some(script);

        Ok(())
    }

    pub async fn reload(&mut self) -> Result<(), LuaVMError> {
        let data = self.data.lock().await;
        if data.file_path.is_none() {
            return Err(LuaVMError::NotLoaded);
        }
        let file_path = data.file_path.clone().unwrap();
        drop(data);

        self.load_file(file_path).await?;
        self.run().await?;

        Ok(())
    }
}
