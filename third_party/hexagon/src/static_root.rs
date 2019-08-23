use std::prelude::v1::*;
use std::any::Any;
use std::cell::RefCell;
use object::Object;

pub struct StaticRoot {
    children: RefCell<Vec<usize>>
}

impl Object for StaticRoot {
    fn get_children(&self) -> Vec<usize> {
        self.children.borrow().clone()
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}

impl StaticRoot {
    pub fn new() -> StaticRoot {
        StaticRoot {
            children: RefCell::new(Vec::new())
        }
    }

    pub fn append_child(&self, id: usize) {
        self.children.borrow_mut().push(id);
    }
}
