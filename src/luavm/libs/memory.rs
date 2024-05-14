use mhw_toolkit::util;
use mlua::prelude::*;
use mlua::UserData;

pub struct Memory;

impl UserData for Memory {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("newPtr", |_, ()| Ok(RawPtr::new()));
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
        methods.add_method_mut("setOffsets", |_, this, offsets: mlua::Variadic<isize>| {
            this.offsets(&offsets);
            Ok(())
        });
        methods.add_method("read", |_, this, type_name: String| {
            let type_name =
                TypeName::from_str(&type_name).ok_or(LuaError::RuntimeError(format!(
                    "Invalid type_name: {}, consider using i32, i64, f32, etc.",
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
                TypeName::String => unimplemented!(),
            }
            .ok_or(LuaError::RuntimeError("Failed to get value".to_string()))
        });
        methods.add_method("write", |_, this, args: mlua::Variadic<mlua::Value>| {
            // :write(value: Value, typeName: Option<String>)
            if args.len() < 1 {
                return Err(LuaError::RuntimeError("At least one parameter must be provided. The current number of parameters is 0".to_string()));
            }
            if args.len() < 2 {
                // only value is provided
                let value = args.first().unwrap();
                return match value {
                    LuaValue::Boolean(v) => {
                        this.set_value(*v).map_err(|e| LuaError::RuntimeError(e.to_string()))
                    },
                    // Integer values support only integers, while Number values support both integers and floats.
                    LuaValue::Integer(_) => Err(LuaError::RuntimeError("Integer value must provide `typeName` argument, such as i32, i64, etc.".to_string())),
                    LuaValue::Number(_) => Err(LuaError::RuntimeError("Number value must provide `typeName` argument, such as i32, f32, etc.".to_string())),
                    LuaValue::String(_) => unimplemented!(),
                    _ => Err(LuaError::RuntimeError(format!("Unsupported value type: {}", value.type_name())))
                };
            }
            if args.len() < 3 {
                // value + typeName
                let value = args.first().unwrap();
                let type_name_value = args.get(1).unwrap();
                let type_name = type_name_value.to_string().map_err(|_| LuaError::RuntimeError("`typeName` must be able to be converted to string".to_string()))?;
                let type_name = TypeName::from_str(&type_name).ok_or(LuaError::RuntimeError(format!(
                    "Invalid type_name: {}, consider using i32, i64, f32, etc.",
                    type_name
                )))?;
                return match value {
                    LuaValue::Boolean(v) => {
                        this.set_value::<bool>(*v).map_err(|e| LuaError::RuntimeError(e.to_string()))
                    },
                    // Integer values support only integers, while Number values support both integers and floats.
                    LuaValue::Integer(v) => {
                        match type_name {
                            TypeName::I8 => this.set_value(*v as i8).map_err(|e| LuaError::RuntimeError(e.to_string())),
                            TypeName::I16 => this.set_value(*v as i16).map_err(|e| LuaError::RuntimeError(e.to_string())),
                            TypeName::I32 => this.set_value(*v as i32).map_err(|e| LuaError::RuntimeError(e.to_string())),
                            TypeName::I64 => this.set_value(*v).map_err(|e| LuaError::RuntimeError(e.to_string())),
                            _ => Err(LuaError::RuntimeError(format!("The type of the value is {:?}, while `typeName` is {:?}, does not match", value ,type_name))),
                        }
                    },
                    LuaValue::Number(v) => {
                        match type_name {
                            TypeName::I8 => this.set_value(*v as i8).map_err(|e| LuaError::RuntimeError(e.to_string())),
                            TypeName::I16 => this.set_value(*v as i16).map_err(|e| LuaError::RuntimeError(e.to_string())),
                            TypeName::I32 => this.set_value(*v as i32).map_err(|e| LuaError::RuntimeError(e.to_string())),
                            TypeName::I64 => this.set_value(*v as i64).map_err(|e| LuaError::RuntimeError(e.to_string())),
                            TypeName::F32 => this.set_value(*v as f32).map_err(|e| LuaError::RuntimeError(e.to_string())),
                            TypeName::F64 => this.set_value(*v).map_err(|e| LuaError::RuntimeError(e.to_string())),
                            _ => Err(LuaError::RuntimeError(format!("The type of the value is {:?}, while `typeName` is {:?}, does not match", value ,type_name))),
                        }
                    },
                    LuaValue::String(_) => unimplemented!(),
                    _ => Err(LuaError::RuntimeError(format!("Unsupported value type: {}", value.type_name())))
                };
            }

            Ok(())
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
