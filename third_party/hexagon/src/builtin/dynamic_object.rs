use std::prelude::v1::*;
use std::any::Any;
use std::collections::HashMap;
use std::cell::{Cell, RefCell};
use object::Object;
use object_pool::ObjectPool;
use value::{Value, ValueContext};
use executor::ExecutorImpl;
use errors::{VMError, RuntimeError};

pub struct DynamicObject {
    prototype: Option<usize>,
    fields: RefCell<HashMap<String, Value>>,
    frozen: Cell<bool>
}

impl Object for DynamicObject {
    fn get_children(&self) -> Vec<usize> {
        let mut children: Vec<usize> = self.fields.borrow().iter().map(|(_, v)| {
            if let Value::Object(id) = *v {
                Some(id)
            } else {
                None
            }
        }).filter(|v| if v.is_some() { true } else { false }).map(|v| v.unwrap()).collect();
        if let Some(prototype) = self.prototype {
            children.push(prototype);
        }
        children
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }

    fn get_field(&self, pool: &ObjectPool, name: &str) -> Option<Value> {
        if let Some(v) = self.fields.borrow().get(name) {
            Some(*v)
        } else {
            if let Some(prototype) = self.prototype {
                let pt_object = pool.get_direct(prototype);
                pt_object.get_field(pool, name)
            } else {
                None
            }
        }
    }

    fn has_const_field(&self, pool: &ObjectPool, name: &str) -> bool {
        if self.frozen.get() {
            true
        } else {
            if let Some(prototype) = self.prototype {
                let pt_object = pool.get_direct(prototype);
                pt_object.has_const_field(pool, name)
            } else {
                false
            }
        }
    }

    fn set_field(&self, name: &str, value: Value) {
        if self.frozen.get() {
            panic!(VMError::from("Attempting to set field on a frozen dynamic object"));
        }
        self.fields.borrow_mut().insert(name.to_string(), value);
    }

    fn call(&self, executor: &mut ExecutorImpl) -> Value {
        let target = match self.get_field(executor.get_object_pool(), "__call__") {
            Some(v) => v,
            None => panic!(VMError::from(RuntimeError::new(
                "Attempting to call a dynamic object without the `__call__` method"
            )))
        };
        let target = ValueContext::new(&target, executor.get_object_pool()).as_object();
        target.call(executor)
    }
}

impl DynamicObject {
    pub fn new(prototype: Option<usize>) -> DynamicObject {
        DynamicObject {
            prototype: prototype,
            fields: RefCell::new(HashMap::new()),
            frozen: Cell::new(false)
        }
    }

    pub fn freeze(&self) {
        self.frozen.set(true);
    }
}
