use mlua::prelude::*;
use mlua::{UserData, Value, Variadic};

pub struct Hook;

impl UserData for Hook {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("setup", |_, this, args: Variadic<Value>| {
            // TODO
            Ok(())
        });
    }
}
