use std::prelude::v1::*;
use std::any::Any;
use std::cmp::Ordering;
use object::Object;
use value::{Value, ValueContext};
use executor::ExecutorImpl;
use errors::{VMError, FieldNotFoundError};

impl Object for String {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn typename(&self) -> &str {
        "string"
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }

    fn to_i64(&self) -> i64 {
        match self.as_str().parse::<i64>() {
            Ok(v) => v,
            Err(_) => panic!(VMError::from("Cannot parse as i64"))
        }
    }

    fn to_f64(&self) -> f64 {
        match self.as_str().parse::<f64>() {
            Ok(v) => v,
            Err(_) => panic!(VMError::from("Cannot parse as f64"))
        }
    }

    fn to_str(&self) -> &str {
        self.as_str()
    }

    fn to_bool(&self) -> bool {
        *self == ""
    }

    fn test_eq(&self, other: &ValueContext) -> bool {
        if let Some(other) = other.as_object_direct().as_any().downcast_ref::<Self>() {
            *other == *self
        } else {
            false
        }
    }

    fn compare(&self, other: &ValueContext) -> Option<Ordering> {
        if let Some(other) = other.as_object_direct().as_any().downcast_ref::<Self>() {
            self.partial_cmp(&other)
        } else {
            None
        }
    }

    fn call_field(&self, field_name: &str, executor: &mut ExecutorImpl) -> Value {
        match field_name {
            "__add__" => {
                let right = executor.get_current_frame().must_get_argument(0);
                let ret = self.clone() + ValueContext::new(&right, executor.get_object_pool()).to_str().as_ref();

                Value::Object(
                    executor.get_object_pool_mut().allocate(
                        Box::new(ret)
                    )
                )
            },
            _ => panic!(VMError::from(FieldNotFoundError::from_field_name(field_name)))
        }
    }
}
