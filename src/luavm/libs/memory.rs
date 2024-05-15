use mhw_toolkit::util;
use mlua::prelude::*;
use mlua::UserData;

pub struct Memory;

impl UserData for Memory {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("newPtr", |_, ()| Ok(RawPtr::new()));
        methods.add_function("read", |_, (addr, type_name): (usize, String)| {
            let type_name =
                TypeName::from_str(&type_name).ok_or(LuaError::RuntimeError(format!(
                    "Invalid typeName: {}, consider using i32, i64, f32, etc.",
                    type_name
                )))?;
            unsafe {
                Ok(match type_name {
                    TypeName::I8 => LuaValue::Integer(*(addr as *const i8) as i64),
                    TypeName::I16 => LuaValue::Integer(*(addr as *const i16) as i64),
                    TypeName::I32 => LuaValue::Integer(*(addr as *const i32) as i64),
                    TypeName::I64 => LuaValue::Integer(*(addr as *const i64)),
                    TypeName::F32 => LuaValue::Number(*(addr as *const f32) as f64),
                    TypeName::F64 => LuaValue::Number(*(addr as *const f64)),
                    TypeName::Bool => LuaValue::Boolean(*(addr as *const bool)),
                    TypeName::String => todo!(),
                })
            }
        });
        methods.add_function(
            "write",
            |_, (addr, value, type_name): (usize, LuaValue, String)| {
                let type_name =
                    TypeName::from_str(&type_name).ok_or(LuaError::RuntimeError(format!(
                        "Invalid typeName: {}, consider using i32, i64, f32, etc.",
                        type_name
                    )))?;
                unsafe {
                    match type_name {
                        TypeName::I8 => *(addr as *mut i8) = value.as_i32().unwrap() as i8,
                        TypeName::I16 => *(addr as *mut i16) = value.as_i32().unwrap() as i16,
                        TypeName::I32 => *(addr as *mut i32) = value.as_i32().unwrap(),
                        TypeName::I64 => *(addr as *mut i64) = value.as_i64().unwrap(),
                        TypeName::F32 => *(addr as *mut f32) = value.as_f32().unwrap(),
                        TypeName::F64 => *(addr as *mut f64) = value.as_f64().unwrap(),
                        TypeName::Bool => *(addr as *mut bool) = value.as_boolean().unwrap(),
                        TypeName::String => todo!(),
                    };
                }
                Ok(())
            },
        );
        methods.add_function("offset", |_, (base, offsets): (usize, Vec<isize>)| {
            util::get_ptr_with_offset(base as *const u8, &offsets)
                .map(|ptr| ptr as usize)
                .ok_or(LuaError::runtime(
                    "Failed to get reference to memory".to_string(),
                ))
        });
    }
}

/// RawPtr provides a reference of a specified memory
pub struct RawPtr {
    base: usize,
    offsets: Vec<isize>,
}

impl UserData for RawPtr {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function_mut("withBase", |_, (ud, base): (LuaAnyUserData, usize)| {
            {
                let mut this = ud.borrow_mut::<RawPtr>()?;
                this.set_base(base);
            }
            Ok(ud)
        });
        methods.add_function_mut(
            "withOffset",
            |_, (ud, offsets): (LuaAnyUserData, mlua::Variadic<isize>)| {
                {
                    let mut this = ud.borrow_mut::<RawPtr>()?;
                    this.offsets(&offsets);
                }
                Ok(ud)
            },
        );
        methods.add_method("read", |_, this, type_name: String| {
            let type_name =
                TypeName::from_str(&type_name).ok_or(LuaError::RuntimeError(format!(
                    "Invalid typeName: {}, consider using i32, i64, f32, etc.",
                    type_name
                )))?;
            match type_name {
                TypeName::I8 => this
                    .get_copy::<i8>()
                    .map(|v| v as i64)
                    .map(mlua::Value::Integer),
                TypeName::I16 => this
                    .get_copy::<i16>()
                    .map(|v| v as i64)
                    .map(mlua::Value::Integer),
                TypeName::I32 => this
                    .get_copy::<i32>()
                    .map(|v| v as i64)
                    .map(mlua::Value::Integer),
                TypeName::I64 => this.get_copy::<i64>().map(mlua::Value::Integer),
                TypeName::F32 => this
                    .get_copy::<f32>()
                    .map(|v| v as f64)
                    .map(mlua::Value::Number),
                TypeName::F64 => this.get_copy::<f64>().map(mlua::Value::Number),
                TypeName::Bool => this.get_copy::<bool>().map(mlua::Value::Boolean),
                TypeName::String => todo!(),
            }
            .ok_or(LuaError::RuntimeError("Failed to get value".to_string()))
        });
        methods.add_method(
            "readMulti",
            |_, this, (type_name, count): (String, usize)| {
                let type_name =
                    TypeName::from_str(&type_name).ok_or(LuaError::RuntimeError(format!(
                        "Invalid typeName: {}, consider using i32, i64, f32, etc.",
                        type_name
                    )))?;
                match type_name {
                    TypeName::I8 => this.get_multi_copy::<i8>(count).map(|v| {
                        v.into_iter()
                            .map(|v| v as i64)
                            .map(mlua::Value::Integer)
                            .collect::<Vec<_>>()
                    }),
                    TypeName::I16 => this.get_multi_copy::<i16>(count).map(|v| {
                        v.into_iter()
                            .map(|v| v as i64)
                            .map(mlua::Value::Integer)
                            .collect::<Vec<_>>()
                    }),
                    TypeName::I32 => this.get_multi_copy::<i32>(count).map(|v| {
                        v.into_iter()
                            .map(|v| v as i64)
                            .map(mlua::Value::Integer)
                            .collect::<Vec<_>>()
                    }),
                    TypeName::I64 => this
                        .get_multi_copy::<i64>(count)
                        .map(|v| v.into_iter().map(mlua::Value::Integer).collect::<Vec<_>>()),
                    TypeName::F32 => this.get_multi_copy::<f32>(count).map(|v| {
                        v.into_iter()
                            .map(|v| v as f64)
                            .map(mlua::Value::Number)
                            .collect::<Vec<_>>()
                    }),
                    TypeName::F64 => this
                        .get_multi_copy::<f64>(count)
                        .map(|v| v.into_iter().map(mlua::Value::Number).collect::<Vec<_>>()),
                    TypeName::Bool => this
                        .get_multi_copy::<bool>(count)
                        .map(|v| v.into_iter().map(mlua::Value::Boolean).collect::<Vec<_>>()),
                    TypeName::String => todo!(),
                }
                .ok_or(LuaError::RuntimeError("Failed to get value".to_string()))
            },
        );
        methods.add_method("write", |_, this, (value, type_name): (mlua::Value, Option<String>)| {
            match type_name {
                Some(type_name) => {
                    let type_sig = TypeName::from_str(&type_name).ok_or(LuaError::RuntimeError(format!("Invalid type name: {}", type_name)))?;
                    match value {
                        LuaValue::Boolean(v) => {
                            this.set_value::<bool>(v).map_err(|e| LuaError::RuntimeError(e.to_string()))
                        },
                        // Integer values support only integers, while Number values support both integers and floats.
                        LuaValue::Integer(v) => {
                            match type_sig {
                                TypeName::I8 => this.set_value(v as i8).map_err(|e| LuaError::RuntimeError(e.to_string())),
                                TypeName::I16 => this.set_value(v as i16).map_err(|e| LuaError::RuntimeError(e.to_string())),
                                TypeName::I32 => this.set_value(v as i32).map_err(|e| LuaError::RuntimeError(e.to_string())),
                                TypeName::I64 => this.set_value(v).map_err(|e| LuaError::RuntimeError(e.to_string())),
                                _ => Err(LuaError::RuntimeError(format!("The type of the value is {:?}, while `typeName` is {:?}, does not match", value ,type_name))),
                            }
                        },
                        LuaValue::Number(v) => {
                            match type_sig {
                                TypeName::I8 => this.set_value(v as i8).map_err(|e| LuaError::RuntimeError(e.to_string())),
                                TypeName::I16 => this.set_value(v as i16).map_err(|e| LuaError::RuntimeError(e.to_string())),
                                TypeName::I32 => this.set_value(v as i32).map_err(|e| LuaError::RuntimeError(e.to_string())),
                                TypeName::I64 => this.set_value(v as i64).map_err(|e| LuaError::RuntimeError(e.to_string())),
                                TypeName::F32 => this.set_value(v as f32).map_err(|e| LuaError::RuntimeError(e.to_string())),
                                TypeName::F64 => this.set_value(v).map_err(|e| LuaError::RuntimeError(e.to_string())),
                                _ => Err(LuaError::RuntimeError(format!("The type of the value is {:?}, while `typeName` is {:?}, does not match", value ,type_name))),
                            }
                        },
                        LuaValue::String(_) => unimplemented!(),
                        _ => Err(LuaError::RuntimeError(format!("Unsupported value type: {}", value.type_name())))
                    }
                }
                None => {
                    match value {
                        LuaValue::Boolean(v) => {
                            this.set_value(v).map_err(|e| LuaError::RuntimeError(e.to_string()))
                        },
                        // Integer values support only integers, while Number values support both integers and floats.
                        LuaValue::Integer(_) => Err(LuaError::RuntimeError("Integer value must provide `typeName` argument, such as i32, i64, etc.".to_string())),
                        LuaValue::Number(_) => Err(LuaError::RuntimeError("Number value must provide `typeName` argument, such as i32, f32, etc.".to_string())),
                        LuaValue::String(_) => todo!(),
                        _ => Err(LuaError::RuntimeError(format!("Unsupported value type: {}", value.type_name())))
                    }
                }
            }
        });
        methods.add_function("clone", |_, this: LuaAnyUserData| Ok(this.clone()));
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

    pub fn get_ptr<T>(&self) -> Option<*const T> {
        util::get_ptr_with_offset(self.base as *const T, &self.offsets)
    }

    pub fn get_multi_copy<T>(&self, count: usize) -> Option<Vec<T>>
    where
        T: Copy,
    {
        let mut result = Vec::with_capacity(count as usize);
        let ptr = self.get_ptr::<T>()?;
        unsafe {
            for i in 0..count {
                result.push(*(ptr.byte_add(std::mem::size_of::<T>() * i)))
            }
        }
        Some(result)
    }

    pub fn set_value<T>(&self, value: T) -> Result<(), String> {
        let ptr = util::get_ptr_with_offset(self.base as *const T, &self.offsets)
            .ok_or("Failed to get reference to memory".to_string())?;
        unsafe {
            *ptr.cast_mut() = value;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TypeName {
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
}

impl TypeName {
    pub fn from_str(type_name: &str) -> Option<TypeName> {
        match type_name {
            "i8" => Some(TypeName::I8),
            "i16" => Some(TypeName::I16),
            "i32" => Some(TypeName::I32),
            "i64" => Some(TypeName::I64),
            "f32" => Some(TypeName::F32),
            "f64" => Some(TypeName::F64),
            "bool" => Some(TypeName::Bool),
            "string" => Some(TypeName::String),
            _ => None,
        }
    }
}
