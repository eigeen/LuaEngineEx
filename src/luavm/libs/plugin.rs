use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use log::error;
use mlua::prelude::*;
use mlua::UserData;
use rand::RngCore;

use crate::luavm::SharedLua;

type EventFuncs = Vec<(u64, mlua::RegistryKey)>;

pub struct Plugin {
    /// event callback functions
    event_listeners: Arc<Mutex<HashMap<EventType, EventFuncs>>>,

    lua: SharedLua,
}

impl UserData for Plugin {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, _| Ok(env!("CARGO_PKG_NAME")));
        fields.add_field_method_get("version", |_, _| Ok(env!("CARGO_PKG_VERSION")));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "addEventListener",
            |lua, this, (event_type_name, f): (String, mlua::Function)| {
                let event_type = EventType::from_str(&event_type_name).ok_or(LuaError::runtime(
                    format!("Invalid event type: {}", event_type_name),
                ))?;
                let func_reg_key = lua.create_registry_value(f)?;
                let id = rand::thread_rng().next_u64();
                this.event_listeners
                    .lock()
                    .unwrap()
                    .entry(event_type)
                    .or_default()
                    .push((id, func_reg_key));

                Ok(id)
            },
        )
    }
}

impl Clone for Plugin {
    fn clone(&self) -> Self {
        Self {
            event_listeners: self.event_listeners.clone(),
            lua: self.lua.clone(),
        }
    }
}

impl Plugin {
    pub fn new(lua: SharedLua) -> Plugin {
        Plugin {
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
            lua,
        }
    }

    pub fn dispatch_standalone_tick(&self) -> Result<(), mlua::Error> {
        if let Some(event_funcs) = self
            .event_listeners
            .lock()
            .unwrap()
            .get(&EventType::StandaloneTick)
        {
            if event_funcs.is_empty() {
                return Ok(());
            }

            let lua_ = self.lua.lock().unwrap();
            for (_, func_reg_key) in event_funcs {
                let f: mlua::Function = lua_.registry_value(func_reg_key)?;
                f.call::<_, ()>(())?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EventType {
    StandaloneTick,
}

impl EventType {
    pub fn from_str(s: &str) -> Option<EventType> {
        match s {
            "StandaloneTick" => Some(EventType::StandaloneTick),
            _ => None,
        }
    }
}

pub fn start_standalone_ticker(p: Plugin) {
    thread::spawn(move || loop {
        if let Err(e) = p.dispatch_standalone_tick() {
            error!("Error in standalone ticker: {}", e);
            return;
        }
        thread::sleep(Duration::from_millis((1000.0 / 60.0) as u64))
    });
}
