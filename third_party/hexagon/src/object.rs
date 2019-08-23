use std::prelude::v1::*;
use std::any::Any;
use std::cmp::Ordering;
use errors;
use executor::{ExecutorImpl};
use object_pool::ObjectPool;
use value::{Value, ValueContext};

/// A heap-allocated object.
///
/// This is the core abstraction in the Hexagon VM type system.
/// After creating, an object must be pinned to the object pool
/// for use in the virtual environment.
///
/// If any initialization is required before the object is pinned
/// to object pool, it should be done in the `initialize` method,
/// which takes a mutable reference to the object pool and makes
/// it possible to do preparations e.g. creating built-in fields.
pub trait Object: Send {
    fn finalize(&self, _pool: &mut ObjectPool) {}

    // before allocating on the object pool...
    fn initialize(&mut self, _pool: &mut ObjectPool) {}

    fn call(&self, _executor: &mut ExecutorImpl) -> Value {
        panic!(errors::VMError::from(errors::RuntimeError::new("Not callable")));
    }
    fn call_field(&self, field_name: &str, executor: &mut ExecutorImpl) -> Value {
        let field = self.must_get_field(executor.get_object_pool(), field_name);
        let obj = ValueContext::new(&field, executor.get_object_pool()).as_object();
        obj.call(executor)
    }
    fn get_field(&self, _pool: &ObjectPool, _name: &str) -> Option<Value> {
        None
    }
    fn set_field(&self, _name: &str, _value_ref: Value) {
        panic!(errors::VMError::from(errors::RuntimeError::new("Cannot set field")));
    }
    fn must_get_field(&self, pool: &ObjectPool, name: &str) -> Value {
        match self.get_field(pool, name) {
            Some(v) => v,
            None => panic!(errors::VMError::from(errors::FieldNotFoundError::from_field_name(name)))
        }
    }
    fn has_const_field(&self, _pool: &ObjectPool, _name: &str) -> bool {
        false
    }
    fn compare(&self, _other: &ValueContext) -> Option<Ordering> {
        None
    }
    fn test_eq(&self, _other: &ValueContext) -> bool {
        false
    }
    fn typename(&self) -> &str {
        "object"
    }
    fn to_i64(&self) -> i64 {
        panic!(errors::VMError::from(errors::RuntimeError::new("Cannot cast to i64")));
    }
    fn to_f64(&self) -> f64 {
        panic!(errors::VMError::from(errors::RuntimeError::new("Cannot cast to f64")));
    }
    fn to_str(&self) -> &str {
        panic!(errors::VMError::from(errors::RuntimeError::new("Cannot cast to str")));
    }
    fn to_string(&self) -> String {
        self.to_str().to_string()
    }
    fn to_bool(&self) -> bool {
        panic!(errors::VMError::from(errors::RuntimeError::new("Cannot cast to bool")));
    }
    fn get_children(&self) -> Vec<usize>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
