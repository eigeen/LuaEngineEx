mod game;
mod hook;
mod memory;
mod plugin;
mod print;

use mlua::prelude::*;

use super::SharedLua;

pub fn load_libs(lua: SharedLua) -> LuaResult<()> {
    let lua_ = lua.lock().unwrap();
    let globals = lua_.globals();
    // print
    globals.set("Print", lua_.create_function(print::fn_info)?)?;
    globals.set("Info", lua_.create_function(print::fn_info)?)?;
    globals.set("Debug", lua_.create_function(print::fn_debug)?)?;
    globals.set("Warn", lua_.create_function(print::fn_warn)?)?;
    globals.set("Error", lua_.create_function(print::fn_error)?)?;
    // plugin system
    let module_plugin = plugin::Plugin::new(lua.clone());
    globals.set(
        "Plugin",
        lua_.create_userdata(module_plugin.clone())?,
    )?;
    // hooks
    globals.set("Hook", lua_.create_userdata(hook::Hook)?)?;
    // memory
    globals.set("Memory", lua_.create_userdata(memory::Memory)?)?;
    // game
    globals.set("Game", lua_.create_userdata(game::Game)?)?;

    // start libs
    plugin::start_standalone_ticker(module_plugin);

    Ok(())
}
