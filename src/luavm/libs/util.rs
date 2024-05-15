// use std::collections::VecDeque;

use mlua::prelude::*;
use mlua::UserData;

pub struct Util;

impl UserData for Util {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {}

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {}
}

// pub struct Queue<'a> {
//     inner: VecDeque<LuaValue<'a>>,
// }

// impl<'a> UserData for Queue<'a> {
//     fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {}

//     fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {}
// }

// impl<'a> Queue<'a> {
//     pub fn new() -> Self {
//         Queue {
//             inner: VecDeque::new(),
//         }
//     }
// }
