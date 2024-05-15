use log::{debug, error, info, warn};
use mlua::prelude::*;
use mlua::Variadic;

pub fn fn_debug(_: &Lua, args: Variadic<mlua::Value>) -> LuaResult<()> {
    debug!(
        "{}",
        args.iter()
            .map(|arg| display_value(arg))
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

pub fn fn_info(_: &Lua, args: Variadic<mlua::Value>) -> LuaResult<()> {
    info!(
        "{}",
        args.iter()
            .map(|arg| display_value(arg))
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

pub fn fn_warn(_: &Lua, args: Variadic<mlua::Value>) -> LuaResult<()> {
    warn!(
        "{}",
        args.iter()
            .map(|arg| display_value(arg))
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

pub fn fn_error(_: &Lua, args: Variadic<mlua::Value>) -> LuaResult<()> {
    error!(
        "{}",
        args.iter()
            .map(|arg| display_value(arg))
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

fn display_value(value: &mlua::Value) -> String {
    match value {
        LuaNil => "nil".to_string(),
        LuaValue::Boolean(b) => format!("{b}"),
        LuaValue::LightUserData(ud) => format!("{ud:?}"),
        LuaValue::Integer(i) => format!("{i}"),
        LuaValue::Number(n) => format!("{n}"),
        LuaValue::String(s) => s.to_str().unwrap_or("<invalid string>").to_string(),
        LuaValue::Table(t) => format!("{t:?}"),
        LuaValue::Function(f) => format!("{f:?}"),
        LuaValue::Thread(t) => format!("{t:?}"),
        LuaValue::UserData(ud) => format!("{ud:?}"),
        LuaValue::Error(e) => format!("{e}"),
    }
}
