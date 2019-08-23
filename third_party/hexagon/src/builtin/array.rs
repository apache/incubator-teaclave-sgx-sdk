use std::prelude::v1::*;
use std::any::Any;
use std::cell::RefCell;
use object::Object;
use value::{Value, ValueContext};
use executor::ExecutorImpl;
use errors::{VMError, FieldNotFoundError};

pub struct Array {
    pub elements: RefCell<Vec<Value>>
}

impl Array {
    pub fn new() -> Array {
        Array {
            elements: RefCell::new(Vec::new())
        }
    }
}

impl Object for Array {
    fn get_children(&self) -> Vec<usize> {
        self.elements.borrow().iter().filter(|v| v.is_object()).map(|v| v.as_object_id()).collect()
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
                let elements = self.elements.borrow();
                if index >= elements.len() {
                    panic!(VMError::from("Array index out of bound"))
                }
                elements[index]
            },
            "__set__" | "set" => {
                let index = executor.get_current_frame().must_get_argument(0);
                let val = executor.get_current_frame().must_get_argument(1);

                let index = ValueContext::new(
                    &index,
                    executor.get_object_pool()
                ).to_i64() as usize;
                let mut elements = self.elements.borrow_mut();

                if index >= elements.len() {
                    panic!(VMError::from("Array index out of bound"))
                }

                (*elements)[index] = val;
                Value::Null
            },
            "push" => {
                let val = executor.get_current_frame().must_get_argument(0);
                self.elements.borrow_mut().push(val);
                Value::Null
            },
            "pop" => {
                self.elements.borrow_mut().pop().unwrap_or_else(|| panic!(VMError::from("No elements")))
            },
            "__len__" | "len" | "size" => {
                Value::Int(self.elements.borrow().len() as i64)
            },
            _ => panic!(VMError::from(FieldNotFoundError::from_field_name(name)))
        }
    }
}
