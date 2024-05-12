use mlua::prelude::*;
use mlua::UserData;

pub struct Plugin;

impl UserData for Plugin {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, _| Ok(env!("CARGO_PKG_NAME")));
        fields.add_field_method_get("version", |_, _| Ok(env!("CARGO_PKG_VERSION")));
    }
}
