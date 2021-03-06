use super::Value;
use std;

impl From<u64> for Value {
    fn from(val: u64) -> Self {
        Value::Uint(val)
    }
}

impl From<f64> for Value {
    fn from(val: f64) -> Self {
        Value::Float(val)
    }
}

impl From<String> for Value {
    fn from(val: String) -> Self {
        Value::Bytes(val.into_bytes())
    }
}

impl<'a> From<&'a str> for Value {
    fn from(val: &'a str) -> Self {
        Value::String(val.to_string())
    }
}

impl<T> From<Option<T>> for Value
where
    Value: std::convert::From<T>,
{
    fn from(val: Option<T>) -> Self {
        match val {
            Some(x) => Value::from(x),
            None => Value::Null,
        }
    }
}

macro_rules! impl_from_value_for_nums {
    ($t:ty) => {
        impl From<Value> for $t {
            fn from(val: Value) -> Self {
                match val {
                    Value::Int(a) => a as Self,
                    Value::Uint(a) => a as Self,
                    Value::Float(a) => a as Self,
                    _ => panic!("Failed to convert to u64"),
                }
            }
        }
    };
}

impl_from_value_for_nums!(usize);
impl_from_value_for_nums!(u64);
impl_from_value_for_nums!(u32);
impl_from_value_for_nums!(i64);
impl_from_value_for_nums!(i32);

impl From<Value> for String {
    fn from(val: Value) -> Self {
        match val {
            Value::String(a) => a,
            _ => panic!("Failed to convert to u64"),
        }
    }
}

impl From<Value> for Option<u64> {
    fn from(val: Value) -> Self {
        match val {
            Value::Null => None,
            Value::Int(x) => Some(x as u64),
            Value::Uint(x) => Some(x as u64),
            _ => None,
        }
    }
}
