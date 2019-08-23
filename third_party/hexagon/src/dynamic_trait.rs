use std::prelude::v1::*;
use std::any::Any;
use std::collections::HashMap;
use std::cell::RefCell;
use object::Object;

pub struct DynamicTrait {
    fields: RefCell<HashMap<String, usize>>
}

impl Object for DynamicTrait {
    fn get_children(&self) -> Vec<usize> {
        self.fields.borrow().iter().map(|(_, v)| *v).collect()
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}
