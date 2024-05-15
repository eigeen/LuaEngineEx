use std::sync::Mutex;

use mhw_toolkit::game::hooks::{CallbackPosition, HookHandle, MonsterCtorHook, MonsterDtorHook};
use once_cell::sync::Lazy;

use super::HookError;

static MONSTER_CTOR_HOOK: Lazy<Mutex<MonsterCtorHook>> =
    Lazy::new(|| Mutex::new(MonsterCtorHook::new()));
static MONSTER_DTOR_HOOK: Lazy<Mutex<MonsterDtorHook>> =
    Lazy::new(|| Mutex::new(MonsterDtorHook::new()));
static MONSTERS: Lazy<Mutex<Vec<usize>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn init_monster_hooks() -> Result<(), HookError> {
    MONSTER_CTOR_HOOK
        .lock()
        .unwrap()
        .set_hook(CallbackPosition::Before, |(monster, _, _)| {
            MONSTERS.lock().unwrap().push(monster as usize);
        })
        .map_err(|e| HookError::Hook {
            source: e,
            reason: "Failed to set monster ctor hook".to_string(),
        })?;
    MONSTER_DTOR_HOOK
        .lock()
        .unwrap()
        .set_hook(CallbackPosition::Before, |monster| {
            MONSTERS.lock().unwrap().retain(|&m| m != monster as usize);
        })
        .map_err(|e| HookError::Hook {
            source: e,
            reason: "Failed to set monster dtor hook".to_string(),
        })?;

    Ok(())
}

pub fn get_all_monsters() -> Vec<usize> {
    MONSTERS.lock().unwrap().clone()
}
