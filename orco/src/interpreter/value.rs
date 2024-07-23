use ir::Type;

use super::*;
use std::num::NonZeroU16;

/// Value - object with a type, returned from an interpreter
#[derive(Debug)]
pub struct Value {
    r#type: Type,
    value: Box<dyn std::any::Any>,
}

impl Value {
    /// Get type of the value
    pub fn r#type<'a>(&'a self) -> &'a Type {
        return &self.r#type;
    }

    /// Try to cast the value to a concrete type (only works if the types match exactly)
    pub fn cast<T: TryFrom<Value>>(self) -> Result<T, <T as TryFrom<Value>>::Error> {
        T::try_from(self)
    }

    /// Try to cast the value to a concrete type (only works if the types match exactly)
    pub fn cast_ref<'a, T>(&'a self) -> Result<&'a T, <&'a T as TryFrom<&'a Value>>::Error>
    where
        &'a T: TryFrom<&'a Value>,
    {
        <&T>::try_from(self)
    }
}

macro_rules! impl_value {
    ($type: ty, $orco_type: expr) => {
        impl_value!($type, value, $orco_type, value.r#type == $orco_type);
    };
    ($type: ty, $value: ident, $orco_type: expr, $check: expr) => {
        impl From<$type> for Value {
            fn from($value: $type) -> Self {
                Value {
                    r#type: $orco_type,
                    value: Box::new($value),
                }
            }
        }

        impl TryFrom<Value> for $type {
            type Error = ();

            fn try_from($value: Value) -> Result<Self, Self::Error> {
                if $check {
                    $value.value.downcast().map(|value| *value).map_err(|_| ())
                } else {
                    Err(())
                }
            }
        }

        impl<'a> TryFrom<&'a Value> for &'a $type {
            type Error = ();

            fn try_from($value: &'a Value) -> Result<Self, Self::Error> {
                if $check {
                    $value.value.downcast_ref().ok_or(())
                } else {
                    Err(())
                }
            }
        }
    };
}

impl_value!((), Type::Unit);

impl_value!(i8, Type::Int(NonZeroU16::new(1).unwrap()));
impl_value!(i16, Type::Int(NonZeroU16::new(2).unwrap()));
impl_value!(i32, Type::Int(NonZeroU16::new(4).unwrap()));
impl_value!(i64, Type::Int(NonZeroU16::new(8).unwrap()));
impl_value!(i128, Type::Int(NonZeroU16::new(16).unwrap()));

impl_value!(u8, Type::Unsigned(NonZeroU16::new(1).unwrap()));
impl_value!(u16, Type::Unsigned(NonZeroU16::new(2).unwrap()));
impl_value!(u32, Type::Unsigned(NonZeroU16::new(4).unwrap()));
impl_value!(u64, Type::Unsigned(NonZeroU16::new(8).unwrap()));
impl_value!(u128, Type::Unsigned(NonZeroU16::new(16).unwrap()));

impl_value!(f32, Type::Float(NonZeroU16::new(4).unwrap()));
impl_value!(f64, Type::Float(NonZeroU16::new(8).unwrap()));
impl_value!(
    ir::expression::Function,
    value,
    value.signature.get_type(),
    matches!(value.r#type, Type::FunctionPointer(_, _))
);

impl Value {
    /// Convert [`ir::expression::Constant`] to a value
    pub fn from_constant(constant: ir::expression::Constant) -> Self {
        match constant {
            ir::expression::Constant::Integer { value, r#type, .. } => match r#type {
                ir::Type::Int(size) if size.get() == 1 => Value::from(value as i8),
                ir::Type::Int(size) if size.get() == 2 => Value::from(value as i16),
                ir::Type::Int(size) if size.get() == 4 => Value::from(value as i32),
                ir::Type::Int(size) if size.get() == 8 => Value::from(value as i64),
                ir::Type::Int(size) if size.get() == 16 => Value::from(value as i128),
                ir::Type::Unsigned(size) if size.get() == 1 => Value::from(value as u8),
                ir::Type::Unsigned(size) if size.get() == 2 => Value::from(value as u16),
                ir::Type::Unsigned(size) if size.get() == 4 => Value::from(value as u32),
                ir::Type::Unsigned(size) if size.get() == 8 => Value::from(value as u64),
                ir::Type::Unsigned(size) if size.get() == 16 => Value::from(value),
                invalid => panic!("Invalid integer literal type: {}", invalid),
            },
            ir::expression::Constant::Float { value, r#type, .. } => match r#type {
                ir::Type::Float(size) if size.get() == 4 => Value::from(value as f32),
                ir::Type::Float(size) if size.get() == 8 => Value::from(value as f64),
                invalid => panic!("Invalid floating point literal type: {}", invalid),
            },
            ir::expression::Constant::CString(_, _) => todo!("CString value"),
        }
    }
}
