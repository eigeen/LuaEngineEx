use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use log::error;
use mhw_toolkit::game::hooks::{CallbackPosition, HookHandle, MonsterCtorHook, MonsterDtorHook};
use mlua::prelude::*;
use mlua::UserData;
use rand::RngCore;
use tokio::runtime::Handle;
use tokio::sync::Mutex;

use crate::luavm::LuaVM;
use crate::luavm::WeakLuaVM;

type EventFuncs = Vec<(u64, mlua::RegistryKey)>;

#[derive(Clone)]
pub struct Plugin {
    /// event callback functions \
    /// key: event type, value: Vec<(id, func_reg_key)>
    event_listeners: Arc<Mutex<HashMap<EventType, EventFuncs>>>,
    /// setInterval callback functions \
    /// key: interval(ms), value: Vec<(id, func_reg_key)>
    interval_listeners: Arc<Mutex<HashMap<u64, EventFuncs>>>,

    monster_ctor_hook: Arc<Mutex<MonsterCtorHook>>,
    monster_dtor_hook: Arc<Mutex<MonsterDtorHook>>,

    luavm: WeakLuaVM,
}

impl UserData for Plugin {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, _| Ok(env!("CARGO_PKG_NAME")));
        fields.add_field_method_get("version", |_, _| Ok(env!("CARGO_PKG_VERSION")));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method_mut(
            "addEventListener",
            |lua, this, (event_type_name, f): (String, mlua::Function)| async move {
                let event_type = EventType::from_str(&event_type_name).ok_or(LuaError::runtime(
                    format!("Invalid event type: {}", event_type_name),
                ))?;
                // special case
                match event_type {
                    EventType::OnMonsterCreate => {
                        let mut hook = this.monster_ctor_hook.lock().await;
                        if !hook.is_hooked() {
                            let p = this.clone();
                            let handle = Handle::current();
                            if let Err(e) =
                                hook.set_hook(CallbackPosition::Before, move |(m, _, _)| {
                                    handle.block_on(async {
                                        if let Err(e) = p
                                            .dispatch_event_monster(
                                                EventType::OnMonsterCreate,
                                                m as i64,
                                            )
                                            .await
                                        {
                                            error!("Error in OnMonsterCreate event: {}", e)
                                        };
                                    })
                                })
                            {
                                error!("Error in OnMonsterCreate hook: {}", e)
                            }
                        };
                    }
                    EventType::OnMonsterDestroy => {
                        let mut hook = this.monster_dtor_hook.lock().await;
                        if !hook.is_hooked() {
                            let p = this.clone();
                            let handle = Handle::current();
                            if let Err(e) = hook.set_hook(CallbackPosition::Before, move |m| {
                                handle.block_on(async {
                                    if let Err(e) = p
                                        .dispatch_event_monster(
                                            EventType::OnMonsterDestroy,
                                            m as i64,
                                        )
                                        .await
                                    {
                                        error!("Error in OnMonsterDestroy event: {}", e)
                                    };
                                })
                            }) {
                                error!("Error in OnMonsterDestroy hook: {}", e)
                            }
                        };
                    }
                    _ => (),
                };

                let func_reg_key = lua.create_registry_value(f)?;
                let id = rand::thread_rng().next_u64();
                this.event_listeners
                    .lock()
                    .await
                    .entry(event_type)
                    .or_default()
                    .push((id, func_reg_key));

                Ok(id)
            },
        );
        methods.add_async_method_mut(
            "setInterval",
            |lua, this, (f, interval): (mlua::Function, u64)| async move {
                let func_reg_key = lua.create_registry_value(f)?;
                let id = rand::thread_rng().next_u64();
                let mut listeners = this.interval_listeners.lock().await;
                if listeners.get(&interval).is_none() {
                    start_set_interval(this.clone(), interval);
                };
                listeners
                    .entry(interval)
                    .or_default()
                    .push((id, func_reg_key));

                Ok(id)
            },
        );
    }
}

impl Plugin {
    pub fn new(luavm: WeakLuaVM) -> Plugin {
        Plugin {
            event_listeners: Arc::new(Mutex::new(HashMap::new())),
            interval_listeners: Arc::new(Mutex::new(HashMap::new())),
            luavm,
            monster_ctor_hook: Arc::new(Mutex::new(MonsterCtorHook::new())),
            monster_dtor_hook: Arc::new(Mutex::new(MonsterDtorHook::new())),
        }
    }

    pub fn get_luavm(&self) -> Option<Arc<Mutex<LuaVM>>> {
        self.luavm.upgrade()
    }

    pub async fn dispatch_set_interval(&self, interval: u64) -> Result<(), mlua::Error> {
        if let Some(interval_funcs) = self.interval_listeners.lock().await.get(&interval) {
            if interval_funcs.is_empty() {
                return Ok(());
            }
            if let Some(luavm) = self.get_luavm() {
                let luavm_ = luavm.lock().await;
                if !luavm_.is_running() {
                    return Ok(());
                }
                for (_, func_reg_key) in interval_funcs {
                    let f: mlua::Function = luavm_.lua.registry_value(func_reg_key)?;
                    f.call_async::<_, ()>(()).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn dispatch_event_monster<'a>(
        &self,
        event_type: EventType,
        arg: i64,
    ) -> Result<(), mlua::Error> {
        if let Some(event_funcs) = self.event_listeners.lock().await.get(&event_type) {
            if event_funcs.is_empty() {
                return Ok(());
            }
            if let Some(luavm) = self.get_luavm() {
                let luavm = luavm.lock().await;
                if !luavm.is_running() {
                    return Ok(());
                }
                for (_, func_reg_key) in event_funcs {
                    let f: mlua::Function = luavm.lua.registry_value(func_reg_key)?;
                    f.call_async::<_, ()>(arg).await?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    OnMonsterCreate,
    OnMonsterDestroy,
}

impl EventType {
    pub fn from_str(s: &str) -> Option<EventType> {
        match s {
            "OnMonsterCreate" => Some(EventType::OnMonsterCreate),
            "OnMonsterDestroy" => Some(EventType::OnMonsterDestroy),
            _ => None,
        }
    }
}

pub fn start_set_interval(p: Plugin, interval: u64) {
    let handle = Handle::current();
    thread::spawn(move || {
        handle.block_on(async {
            while p.get_luavm().is_some() {
                if let Err(e) = p.dispatch_set_interval(interval).await {
                    error!("Error in setInterval: {}", e);
                    return;
                }
                tokio::time::sleep(Duration::from_millis(interval)).await;
            }
        })
    });
}
