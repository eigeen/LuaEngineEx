use std::{path::Path, sync::{Arc, Mutex}};

use log::debug;
use mlua::prelude::*;
use snafu::prelude::*;

use super::libs;

pub type SharedLua = Arc<Mutex<Lua>>;

#[derive(Debug, Snafu)]
pub enum LuaVMError {
    #[snafu(display("Failed to load script file: {}", source))]
    LoadFile { source: std::io::Error },
    #[snafu(display("Lua VM runs before loading script"))]
    NotLoaded,
    #[snafu(display("Failed to load script: {}", source))]
    LuaRuntime { source: mlua::Error },
}

#[derive(Debug)]
pub struct LuaVMData {
    pub name: String,
    pub file_path: Option<String>,
    pub script: Option<String>,
}

#[derive(Debug)]
pub struct LuaVM {
    pub data: LuaVMData,
    lua: SharedLua,
}

impl LuaVM {
    pub fn new(name: &str) -> Self {
        Self {
            data: LuaVMData {
                name: name.to_string(),
                file_path: None,
                script: None,
            },
            lua: Arc::new(Mutex::new(Lua::new())),
        }
    }

    fn run_inner(&self, script: &str) -> LuaResult<()> {
        self.load_libs()?;
        self.lua.lock().unwrap().load(script).exec()?;

        Ok(())
    }

    fn load_libs(&self) -> LuaResult<()> {
        libs::load_libs(self.lua.clone())
    }

    pub fn run(&self) -> Result<(), LuaVMError> {
        if self.data.script.is_none() {
            return Err(LuaVMError::NotLoaded);
        }

        let script = &self.data.script.as_ref().unwrap();
        debug!("Lua VM \"{}\" start running", self.data.name);
        self.run_inner(script)
            .map_err(|e| LuaVMError::LuaRuntime { source: e })?;

        Ok(())
    }

    pub fn load_file<P>(&mut self, file_path: P) -> Result<(), LuaVMError>
    where
        P: AsRef<Path>,
    {
        let script = std::fs::read_to_string(&file_path).context(LoadFileSnafu)?;
        self.data.file_path = Some(file_path.as_ref().to_string_lossy().to_string());
        self.data.script = Some(script);

        Ok(())
    }

    pub fn reload(&mut self) -> Result<(), LuaVMError> {
        if self.data.file_path.is_none() {
            return Err(LuaVMError::NotLoaded);
        }
        let file_path = self.data.file_path.clone().unwrap();
        self.load_file(file_path)?;
        self.run()?;

        Ok(())
    }
}
