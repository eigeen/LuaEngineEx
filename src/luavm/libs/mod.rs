mod game;
mod memory;
mod plugin;
mod print;
mod util;

use mlua::prelude::*;

use super::WeakLuaVM;

pub async fn load_libs(luavm: WeakLuaVM) -> LuaResult<()> {
    let luavm_ = luavm.upgrade().unwrap();
    let lua_ = &luavm_.lock().await.lua;
    let globals = lua_.globals();
    // override
    globals.set("Print", lua_.create_function(print::fn_info)?)?;
    globals.set("Info", lua_.create_function(print::fn_info)?)?;
    globals.set("Debug", lua_.create_function(print::fn_debug)?)?;
    globals.set("Warn", lua_.create_function(print::fn_warn)?)?;
    globals.set("Error", lua_.create_function(print::fn_error)?)?;
    {
        let package: LuaTable = globals.get("package")?;
        let path: String = package.get("path")?;
        let new_path = format!("{};./LuaEngineEx/?.lua", path);
        package.set("path", new_path)?;
    }

    // plugin system
    let module_plugin = plugin::Plugin::new(luavm.clone());
    globals.set("Plugin", lua_.create_userdata(module_plugin.clone())?)?;
    // memory
    globals.set("Memory", lua_.create_userdata(memory::Memory)?)?;
    // game
    globals.set("Game", lua_.create_userdata(game::Game)?)?;

    Ok(())
}
