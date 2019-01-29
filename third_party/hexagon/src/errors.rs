use std::prelude::v1::*;
use std::any::Any;
use object::Object;

pub struct VMError {
    inner: Box<Object>
}

impl<T> From<T> for VMError where T: Object + 'static {
    fn from(other: T) -> VMError {
        VMError {
            inner: Box::new(other)
        }
    }
}

impl<'a> From<&'a str> for VMError {
    fn from(other: &'a str) -> VMError {
        VMError {
            inner: Box::new(other.to_string())
        }
    }
}

impl VMError {
    pub fn unwrap(self) -> Box<Object> {
        self.inner
    }
}

pub struct ValidateError {
    description: String
}

impl Object for ValidateError {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn to_str(&self) -> &str {
        self.description.as_str()
    }
}

impl ValidateError {
    pub fn new<T: ToString>(desc: T) -> ValidateError {
        ValidateError {
            description: desc.to_string()
        }
    }
}

pub struct ParseError {
    description: String
}

impl Object for ParseError {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn to_str(&self) -> &str {
        self.description.as_str()
    }
}

impl ParseError {
    pub fn new<T: ToString>(desc: T) -> ParseError {
        ParseError {
            description: desc.to_string()
        }
    }
}

pub struct RuntimeError {
    description: String
}

impl Object for RuntimeError {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn to_str(&self) -> &str {
        self.description.as_str()
    }
}

impl RuntimeError {
    pub fn new<T: ToString>(desc: T) -> RuntimeError {
        RuntimeError {
            description: desc.to_string()
        }
    }
}

pub struct FieldNotFoundError {
    field_name: String
}

impl Object for FieldNotFoundError {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn to_string(&self) -> String {
        format!("Field not found: {}", self.field_name)
    }
}

impl FieldNotFoundError {
    pub fn from_field_name<T: ToString>(name: T) -> FieldNotFoundError {
        FieldNotFoundError {
            field_name: name.to_string()
        }
    }
}
