use super::*;

/// Value - any object returned from an interpreter (or JIT)
#[derive(Debug)]
pub struct Value(Box<dyn std::any::Any + Send + Sync>);

impl Value {}

impl Value {
    /// Create a new value
    pub fn new<T: Send + Sync + 'static>(value: T) -> Self {
        Self(Box::new(value))
    }

    /// Try to cast the value to a concrete type (only works if the types match exactly)
    pub fn r#as<T: 'static>(&self) -> &T {
        self.0.downcast_ref().unwrap_or_else(|| {
            panic!(
                "Pointer cast of dynamic value (type_id {:?}) to an invalid type {}",
                self.0.type_id(),
                std::any::type_name::<T>()
            )
        })
    }

    /// Try to cast the value to a concrete type (only works if the types match exactly)
    pub fn into<T: 'static>(self) -> Box<T> {
        self.0.downcast().unwrap_or_else(|val| {
            panic!(
                "Cast of dynamic value (type_id {:?}) to an invalid type {}",
                val.type_id(),
                std::any::type_name::<T>()
            )
        })
    }
}

impl Value {
    /// Convert [`ir::expression::Constant`] to a value
    pub fn from_constant(constant: ir::expression::Constant) -> Self {
        match constant {
            ir::expression::Constant::Integer { value, r#type, .. } => match r#type {
                ir::Type::Int(size) if size.get() == 1 => Value::new(value as i8),
                ir::Type::Int(size) if size.get() == 2 => Value::new(value as i16),
                ir::Type::Int(size) if size.get() == 4 => Value::new(value as i32),
                ir::Type::Int(size) if size.get() == 8 => Value::new(value as i64),
                ir::Type::Int(size) if size.get() == 16 => Value::new(value as i128),
                ir::Type::Unsigned(size) if size.get() == 1 => Value::new(value as u8),
                ir::Type::Unsigned(size) if size.get() == 2 => Value::new(value as u16),
                ir::Type::Unsigned(size) if size.get() == 4 => Value::new(value as u32),
                ir::Type::Unsigned(size) if size.get() == 8 => Value::new(value as u64),
                ir::Type::Unsigned(size) if size.get() == 16 => Value::new(value),
                invalid => panic!("Invalid integer literal type: {}", invalid),
            },
            ir::expression::Constant::Float { value, r#type, .. } => match r#type {
                ir::Type::Float(size) if size.get() == 4 => Value::new(value as f32),
                ir::Type::Float(size) if size.get() == 8 => Value::new(value),
                invalid => panic!("Invalid floating point literal type: {}", invalid),
            },
            ir::expression::Constant::CString(_, _) => todo!("CString value"),
        }
    }
}
