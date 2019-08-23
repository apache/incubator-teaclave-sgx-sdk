use std::prelude::v1::*;
use std::any::Any;
use std::cell::UnsafeCell;
use object::Object;
use executor::ExecutorImpl;
use value::{Value, ValueContext};
use errors::{VMError, FieldNotFoundError};

pub trait TypedArrayElement: Send + Copy + 'static {
    fn must_from_value(other: Value) -> Self {
        Self::from_value(other).unwrap_or_else(|| panic!(VMError::from("Invalid cast")))
    }
    fn from_value(other: Value) -> Option<Self>;
    fn to_value(&self) -> Value;
}

pub struct TypedArray<T: TypedArrayElement> {
    elements: UnsafeCell<Vec<T>>,
    default_value: T
}

impl<T: TypedArrayElement> TypedArray<T> {
    pub fn new(value: T, len: usize) -> TypedArray<T> {
        TypedArray {
            elements: UnsafeCell::new(vec![value; len]),
            default_value: value
        }
    }

    pub fn resize(&self, len: usize) {
        let elements = unsafe { &mut *self.elements.get() };
        elements.resize(len, self.default_value);
    }

    pub fn set(&self, id: usize, v: T) {
        let elements = unsafe { &mut *self.elements.get() };
        if id < elements.len() {
            elements[id] = v;
        } else {
            panic!(VMError::from("TypedArray index out of bound"));
        }
    }

    pub fn get(&self, id: usize) -> T {
        let elements = unsafe { &mut *self.elements.get() };
        if id < elements.len() {
            elements[id]
        } else {
            panic!(VMError::from("TypedArray index out of bound"));
        }
    }

    pub fn len(&self) -> usize {
        let elements = unsafe { &mut *self.elements.get() };
        elements.len()
    }
}

impl<T: TypedArrayElement> Object for TypedArray<T> {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }

    fn call_field(&self, name: &str, executor: &mut ExecutorImpl) -> Value {
        match name {
            "__get__" | "get" => {
                let index = executor.get_current_frame().must_get_argument(0);
                let index = ValueContext::new(
                    &index,
                    executor.get_object_pool()
                ).to_i64() as usize;
                self.get(index).to_value()
            },
            "__set__" | "set" => {
                let index = executor.get_current_frame().must_get_argument(0);
                let val = executor.get_current_frame().must_get_argument(1);

                let index = ValueContext::new(
                    &index,
                    executor.get_object_pool()
                ).to_i64() as usize;
                if let Some(v) = T::from_value(val) {
                    self.set(index, v);
                } else {
                    panic!(VMError::from("Cannot cast to target type"));
                }
                Value::Null
            },
            "resize" => {
                let new_size = ValueContext::new(
                    &executor.get_current_frame().must_get_argument(0),
                    executor.get_object_pool()
                ).to_i64() as usize;
                self.resize(new_size);
                Value::Null
            },
            "__len__" | "len" | "size" => {
                Value::Int(self.len() as i64)
            },
            _ => panic!(VMError::from(FieldNotFoundError::from_field_name(name)))
        }
    }
}

macro_rules! impl_typed_int {
    ($type_name:ty) => (
        impl TypedArrayElement for $type_name {
            fn from_value(v: Value) -> Option<Self> {
                match v {
                    Value::Int(v) => {
                        if v >= Self::min_value() as i64 && v <= Self::max_value() as i64 {
                            Some(v as $type_name)
                        } else {
                            None
                        }
                    },
                    Value::Float(v) => {
                        let v = v as i64;
                        if v >= Self::min_value() as i64 && v <= Self::max_value() as i64 {
                            Some(v as $type_name)
                        } else {
                            None
                        }
                    },
                    _ => None
                }
            }

            fn to_value(&self) -> Value {
                Value::Int(*self as i64)
            }
        }
    )
}

impl_typed_int!(i8);
impl_typed_int!(u8);
impl_typed_int!(i16);
impl_typed_int!(u16);
impl_typed_int!(i32);
impl_typed_int!(u32);
impl_typed_int!(i64);
impl_typed_int!(u64);

impl TypedArrayElement for f32 {
    fn from_value(v: Value) -> Option<Self> {
        match v {
            Value::Int(v) => {
                let v = v as f64;
                if v >= ::std::f32::MIN as f64 && v <= ::std::f32::MAX as f64 {
                    Some(v as f32)
                } else {
                    None
                }
            },
            Value::Float(v) => {
                if v >= ::std::f32::MIN as f64 && v <= ::std::f32::MAX as f64 {
                    Some(v as f32)
                } else {
                    None
                }
            },
            _ => None
        }
    }

    fn to_value(&self) -> Value {
        Value::Float(*self as f64)
    }
}

impl TypedArrayElement for f64 {
    fn from_value(v: Value) -> Option<Self> {
        match v {
            Value::Int(v) => {
                Some(v as f64)
            },
            Value::Float(v) => {
                Some(v)
            },
            _ => None
        }
    }

    fn to_value(&self) -> Value {
        Value::Float(*self)
    }
}
