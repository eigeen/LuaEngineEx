use log::{debug, error, info, warn};
use mlua::prelude::*;
use mlua::Variadic;

pub fn fn_debug(_: &Lua, args: Variadic<String>) -> LuaResult<()> {
    debug!("{}", args.join(", "));
    Ok(())
}

pub fn fn_info(_: &Lua, args: Variadic<String>) -> LuaResult<()> {
    info!("{}", args.join(", "));
    Ok(())
}

pub fn fn_warn(_: &Lua, args: Variadic<String>) -> LuaResult<()> {
    warn!("{}", args.join(", "));
    Ok(())
}

pub fn fn_error(_: &Lua, args: Variadic<String>) -> LuaResult<()> {
    error!("{}", args.join(", "));
    Ok(())
}
