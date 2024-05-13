use mhw_toolkit::util;
use mlua::prelude::*;
use mlua::UserData;
use mlua::Variadic;

pub struct Memory;

impl UserData for Memory {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("Ptr", |_, ()| Ok(RawPtr::new()));
    }
}

pub struct RawPtr {
    base: usize,
    offsets: Vec<isize>,
}

impl UserData for RawPtr {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("setBase", |_, this, base: usize| {
            this.set_base(base);
            Ok(())
        });
        methods.add_method_mut("setOffsets", |_, this, offsets: Variadic<isize>| {
            this.offsets(&offsets);
            Ok(())
        });

        methods.add_method("read", |_, this, type_name: String| {
            match type_name.as_str() {
                "i8" => this
                    .get_copy::<i8>()
                    .map(|v| v as i64)
                    .map(mlua::Value::Integer)
                    .ok_or(mlua::Error::RuntimeError("Failed to get value".to_string())),
                "i16" => this
                    .get_copy::<i16>()
                    .map(|v| v as i64)
                    .map(mlua::Value::Integer)
                    .ok_or(mlua::Error::RuntimeError("Failed to get value".to_string())),
                "i32" => this
                    .get_copy::<i32>()
                    .map(|v| v as i64)
                    .map(mlua::Value::Integer)
                    .ok_or(mlua::Error::RuntimeError("Failed to get value".to_string())),
                "i64" => this
                    .get_copy::<i64>()
                    .map(mlua::Value::Integer)
                    .ok_or(mlua::Error::RuntimeError("Failed to get value".to_string())),
                "f32" => this
                    .get_copy::<f32>()
                    .map(|v| v as f64)
                    .map(mlua::Value::Number)
                    .ok_or(mlua::Error::RuntimeError("Failed to get value".to_string())),
                "f64" => this
                    .get_copy::<f64>()
                    .map(mlua::Value::Number)
                    .ok_or(mlua::Error::RuntimeError("Failed to get value".to_string())),
                _ => Err(mlua::Error::RuntimeError(format!(
                    "Invalid type_name: {}, consider using i32, i64, f32, etc.",
                    type_name
                ))),
            }
        });
    }
}

impl Default for RawPtr {
    fn default() -> Self {
        Self::new()
    }
}

impl RawPtr {
    pub fn new() -> RawPtr {
        RawPtr {
            base: 0,
            offsets: Vec::new(),
        }
    }

    pub fn set_base(&mut self, base: usize) {
        self.base = base;
    }

    pub fn offset(&mut self, offset: isize) {
        self.offsets.push(offset);
    }

    pub fn offsets(&mut self, offsets: &[isize]) {
        self.offsets.extend_from_slice(offsets);
    }

    pub fn get_copy<T>(&self) -> Option<T>
    where
        T: Copy,
    {
        util::get_value_with_offset(self.base as *const T, &self.offsets)
    }

    // pub fn set_value(&self, value: T) -> Result<(), ()> {
    //     let ptr = util::get_ptr_with_offset(self.base as *const T, &self.offsets).ok_or(())?;
    //     unsafe {
    //         *ptr.cast_mut() = value;
    //     }

    //     Ok(())
    // }
}
