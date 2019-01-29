use std::prelude::v1::*;
use std::any::Any;
use std::collections::HashMap;
use std::cell::{Cell, RefCell};
use hexagon::object::Object;
use hexagon::value::{Value, ValueContext};
use hexagon::executor::ExecutorImpl;
use hexagon::errors::{VMError, FieldNotFoundError};
use hexagon::builtin::array::Array;

pub struct Pair {
    pub left: Value,
    pub right: Value
}

impl Object for Pair {
    fn get_children(&self) -> Vec<usize> {
        let mut ret: Vec<usize> = Vec::new();
        if let Value::Object(id) = self.left {
            ret.push(id);
        }
        if let Value::Object(id) = self.right {
            ret.push(id);
        }
        ret
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }
}

fn f64_to_u64(v: f64) -> u64 {
    if v.is_nan() {
        panic!(VMError::from("NaN"));
    }
    unsafe { ::std::mem::transmute::<f64, u64>(v) }
}

fn u64_to_f64(v: u64) -> f64 {
    let result = unsafe { ::std::mem::transmute::<u64, f64>(v) };
    if result.is_nan() {
        panic!(VMError::from("NaN"));
    }
    result
}

pub struct Table {
    string_values: RefCell<HashMap<String, Value>>,
    number_values: RefCell<HashMap<u64, Value>> // f64 keys actually
}

macro_rules! do_val_insert {
    ($t:expr, $k:expr, $v:expr) => ({
        if $v == Value::Null {
            $t.remove(&$k);
        } else {
            $t.insert($k, $v);
        }
    })
}

impl Table {
    pub fn new() -> Table {
        Table {
            string_values: RefCell::new(HashMap::new()),
            number_values: RefCell::new(HashMap::new())
        }
    }

    pub fn clear(&self) {
        let mut string_values = self.string_values.borrow_mut();
        let mut number_values = self.number_values.borrow_mut();

        string_values.clear();
        number_values.clear();
    }

    pub fn len(&self) -> usize {
        self.string_values.borrow().len() + self.number_values.borrow().len()
    }

    pub fn set(&self, executor: &mut ExecutorImpl, k: Value, ins_value: Value) {
        match k {
            Value::Int(v) => {
                do_val_insert!(self.number_values.borrow_mut(), f64_to_u64(v as f64), ins_value);
            },
            Value::Float(v) => {
                do_val_insert!(self.number_values.borrow_mut(), f64_to_u64(v), ins_value);
            },
            Value::Object(_) => {
                let k = ValueContext::new(&k, executor.get_object_pool()).to_str().to_string();
                do_val_insert!(self.string_values.borrow_mut(), k, ins_value);
            },
            _ => panic!(VMError::from("Table: Unsupported key"))
        }
    }

    pub fn get(&self, executor: &mut ExecutorImpl, k: Value) -> Value {
        match k {
            Value::Int(v) => {
                *self.number_values.borrow_mut().get(&f64_to_u64(v as f64)).unwrap_or(&Value::Null)
            },
            Value::Float(v) => {
                *self.number_values.borrow_mut().get(&f64_to_u64(v)).unwrap_or(&Value::Null)
            },
            Value::Object(_) => {
                let k = ValueContext::new(&k, executor.get_object_pool());
                *self.string_values.borrow_mut().get(k.to_str().as_ref()).unwrap_or(&Value::Null)
            },
            _ => panic!(VMError::from("Table: Unsupported key"))
        }
    }
}

impl Object for Table {
    fn get_children(&self) -> Vec<usize> {
        let mut ret: Vec<usize> = Vec::new();
        for (_, v) in self.string_values.borrow().iter() {
            if let Value::Object(id) = *v {
                ret.push(id);
            }
        }
        for (_, v) in self.number_values.borrow().iter() {
            if let Value::Object(id) = *v {
                ret.push(id);
            }
        }
        ret
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn call_field(&self, name: &str, executor: &mut ExecutorImpl) -> Value {
        match name {
            "__get__" => {
                let key = executor.get_current_frame().must_get_argument(0);
                self.get(executor, key)
            },
            "__set__" => {
                let key = executor.get_current_frame().must_get_argument(0);
                let value = executor.get_current_frame().must_get_argument(1);
                self.set(executor, key, value);
                Value::Null
            },
            "__len__" => {
                Value::Int(self.len() as i64)
            },
            "__copy_from_array__" => {
                let pool = executor.get_object_pool();

                let array = executor.get_current_frame().must_get_argument(0).as_object_id();
                let array = pool.must_get_direct_typed::<Array>(array);

                self.clear();

                let mut deferred_items: Vec<Value> = Vec::new();
                let mut deferred_pairs: Vec<(Value, Value)> = Vec::new();

                for elem in array.elements.borrow().iter() {
                    if let Value::Object(id) = *elem {
                        if let Some(pair) = pool.get_direct_typed::<Pair>(id) {
                            deferred_pairs.push((pair.left, pair.right));
                            continue;
                        }
                    }
                    deferred_items.push(*elem);
                }

                for (k, v) in deferred_pairs {
                    self.set(executor, k, v);
                }
                for (i, v) in deferred_items.into_iter().enumerate() {
                    self.set(executor, Value::Float((i + 1) as f64), v);
                }

                Value::Null
            },
            _ => panic!(VMError::from(FieldNotFoundError::from_field_name(name)))
        }
    }
}
