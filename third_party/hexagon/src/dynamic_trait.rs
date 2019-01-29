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

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }
}
