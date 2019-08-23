use std::prelude::v1::*;
pub mod array;
pub mod dynamic_object;
pub mod typed_array;

use std::any::Any;
use object::Object;
use function::Function;
use value::{Value, ValueContext};
use executor::ExecutorImpl;
use errors::{VMError, FieldNotFoundError};
use generic_arithmetic;
use self::typed_array::TypedArray;
use self::typed_array::TypedArrayElement;

pub struct BuiltinObject {

}

impl BuiltinObject {
    pub fn new() -> BuiltinObject {
        BuiltinObject {}
    }
}

impl Object for BuiltinObject {
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
            "new_array" => {
                let array_obj: Box<dyn Object> = Box::new(array::Array::new());
                Value::Object(
                    executor.get_object_pool_mut().allocate(array_obj)
                )
            },
            "new_dynamic" => {
                let prototype = match executor.get_current_frame().must_get_argument(0) {
                    Value::Object(id) => Some(id),
                    Value::Null => None,
                    _ => panic!(VMError::from("Invalid prototype object"))
                };
                Value::Object(executor.get_object_pool_mut().allocate(
                    Box::new(dynamic_object::DynamicObject::new(prototype))
                ))
            },
            "freeze_dynamic" => {
                let target_id = match executor.get_current_frame().must_get_argument(0) {
                    Value::Object(id) => id,
                    _ => panic!(VMError::from("Invalid target object"))
                };
                let target: &dynamic_object::DynamicObject = executor.get_object_pool().must_get_direct_typed(target_id);
                target.freeze();
                Value::Null
            },
            "optimize" => {
                let target_id = match executor.get_current_frame().must_get_argument(0) {
                    Value::Object(id) => id,
                    _ => panic!(VMError::from("Invalid target object"))
                };
                let target = executor.get_object_pool().must_get_typed::<Function>(target_id);
                target.dynamic_optimize(executor.get_object_pool_mut());
                Value::Null
            },
            "new_typed_array" => {
                let type_name = ValueContext::new(
                    &executor.get_current_frame().must_get_argument(0),
                    executor.get_object_pool()
                ).to_str().to_string();
                let size = ValueContext::new(
                    &executor.get_current_frame().must_get_argument(1),
                    executor.get_object_pool()
                ).to_i64() as usize;
                let default_value = executor.get_current_frame().must_get_argument(2);

                let obj_id = executor.get_object_pool_mut().allocate(match type_name.as_str() {
                    "i8" => Box::new(TypedArray::new(
                        i8::must_from_value(default_value),
                        size
                    )),
                    "u8" => Box::new(TypedArray::new(
                        u8::must_from_value(default_value),
                        size
                    )),
                    "i16" => Box::new(TypedArray::new(
                        i16::must_from_value(default_value),
                        size
                    )),
                    "u16" => Box::new(TypedArray::new(
                        u16::must_from_value(default_value),
                        size
                    )),
                    "i32" => Box::new(TypedArray::new(
                        i32::must_from_value(default_value),
                        size
                    )),
                    "u32" => Box::new(TypedArray::new(
                        u32::must_from_value(default_value),
                        size
                    )),
                    "i64" => Box::new(TypedArray::new(
                        i64::must_from_value(default_value),
                        size
                    )),
                    "u64" => Box::new(TypedArray::new(
                        u64::must_from_value(default_value),
                        size
                    )),
                    _ => panic!(VMError::from("Unknown type"))
                });
                Value::Object(obj_id)
            },
            "add" => {
                let (left, right) = (executor.get_current_frame().must_get_argument(0), executor.get_current_frame().must_get_argument(1));
                generic_arithmetic::exec_add(executor, left, right)
            },
            "sub" => {
                let (left, right) = (executor.get_current_frame().must_get_argument(0), executor.get_current_frame().must_get_argument(1));
                generic_arithmetic::exec_sub(executor, left, right)
            },
            "mul" => {
                let (left, right) = (executor.get_current_frame().must_get_argument(0), executor.get_current_frame().must_get_argument(1));
                generic_arithmetic::exec_mul(executor, left, right)
            },
            "div" => {
                let (left, right) = (executor.get_current_frame().must_get_argument(0), executor.get_current_frame().must_get_argument(1));
                generic_arithmetic::exec_div(executor, left, right)
            },
            "mod" => {
                let (left, right) = (executor.get_current_frame().must_get_argument(0), executor.get_current_frame().must_get_argument(1));
                generic_arithmetic::exec_mod(executor, left, right)
            },
            "pow" => {
                let (left, right) = (executor.get_current_frame().must_get_argument(0), executor.get_current_frame().must_get_argument(1));
                generic_arithmetic::exec_pow(executor, left, right)
            },
            _ => panic!(VMError::from(FieldNotFoundError::from_field_name(name)))
        }
    }
}
