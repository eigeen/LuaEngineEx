use mhw_toolkit::game_util::ChatMessageSender;
use mhw_toolkit::game_util::SystemMessageColor;
use mlua::prelude::*;
use mlua::UserData;
use once_cell::sync::Lazy;

use crate::hooks;

static CHAT_MESSAGE_SENDER: Lazy<ChatMessageSender> = Lazy::new(ChatMessageSender::new);

pub struct Game;

impl UserData for Game {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Chat", |_, _| Ok(Chat));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("getAllMonsters", |lua, ()| {
            let monsters = hooks::monster::get_all_monsters();
            if monsters.is_empty() {
                return Ok(LuaValue::Nil);
            }

            let len = monsters.len();
            Ok(LuaValue::Table(
                lua.create_table_from(monsters.into_iter().zip(0..len))?,
            ))
        })
    }
}

pub struct Chat;

impl UserData for Chat {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("sendMessage", |_, arg: String| {
            CHAT_MESSAGE_SENDER.send(&arg);
            Ok(())
        });
        methods.add_function(
            "showSystemMessage",
            |_, (msg, color): (String, Option<String>)| {
                let color_value = match color {
                    Some(c) => match c.to_lowercase().as_str() {
                        "blue" | "general" => SystemMessageColor::Blue,
                        "purple" | "primary" => SystemMessageColor::Purple,
                        _ => {
                            return Err(LuaError::runtime(format!(
                                "Unsupported color: {}, expect `blue|general` or `purple|primary`",
                                c
                            )))
                        }
                    },
                    None => SystemMessageColor::Blue,
                };
                mhw_toolkit::game_util::show_system_message(&msg, color_value);

                Ok(())
            },
        );
    }
}
