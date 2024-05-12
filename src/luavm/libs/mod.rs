mod plugin;
mod print;

use mlua::prelude::*;

use self::plugin::Plugin;

pub fn load_libs(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();
    // print
    globals.set("Print", lua.create_function(print::fn_info)?)?;
    globals.set("Info", lua.create_function(print::fn_info)?)?;
    globals.set("Debug", lua.create_function(print::fn_debug)?)?;
    globals.set("Warn", lua.create_function(print::fn_warn)?)?;
    globals.set("Error", lua.create_function(print::fn_error)?)?;
    // plugin metadata
    globals.set("Plugin", lua.create_userdata(Plugin)?)?;

    Ok(())
}
