use std::prelude::v1::*;
use std::cell::Cell;
use std::collections::HashSet;
use smallvec::SmallVec;
use errors;
use value::Value;
use opcode::StackMapPattern;
use object_pool::ObjectPool;

fixed_array!(FixedArray32, 32);

pub struct CallStack {
    frames: Vec<Frame>,
    n_frames: usize,
    limit: Option<usize>
}

pub type FrameHandle = Frame;

// [unsafe]
// These fields are guaranteed to be accessed properly (as an implementation detail).
pub struct Frame {
    this: Cell<Value>,
    arguments: FixedArray32<Value>,
    locals: FixedArray32<Value>,
    pub(crate) exec_stack: FixedArray32<Value>
}

impl CallStack {
    pub fn new(len: usize) -> CallStack {
        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            frames.push(Frame::new());
        }

        CallStack {
            frames: frames,
            n_frames: 1, // one 'initial' frame
            limit: None
        }
    }

    pub fn set_limit(&mut self, limit: usize) {
        self.limit = Some(limit);
    }

    pub fn push(&mut self) {
        if self.n_frames >= self.frames.len() {
            panic!(errors::VMError::from(errors::RuntimeError::new("Virtual stack overflow")));
        }
        if let Some(limit) = self.limit {
            if self.n_frames >= limit {
                panic!(errors::VMError::from(errors::RuntimeError::new("Maximum stack depth exceeded")));
            }
        }
        self.n_frames += 1;
    }

    pub fn pop(&mut self) {
        if self.n_frames <= 0 {
            panic!(errors::VMError::from(errors::RuntimeError::new("Virtual stack underflow")));
        }
        self.frames[self.n_frames - 1].reset();
        self.n_frames -= 1;
    }

    pub fn top(&self) -> &Frame {
        if self.n_frames <= 0 {
            panic!(errors::VMError::from(errors::RuntimeError::new("Virtual stack underflow")));
        }
        &self.frames[self.n_frames - 1]
    }

    pub fn collect_objects(&self) -> Vec<usize> {
        let mut objs = HashSet::new();
        for i in 0..self.n_frames {
            let frame = &self.frames[i];
            if let Value::Object(id) = frame.this.get() {
                objs.insert(id);
            }
            for i in 0..frame.arguments.len() {
                let v = frame.arguments.get(i).unwrap();
                if let Value::Object(id) = v {
                    objs.insert(id);
                }
            }
            for i in 0..frame.locals.len() {
                let v = frame.locals.get(i).unwrap();
                if let Value::Object(id) = v {
                    objs.insert(id);
                }
            }
            for i in 0..frame.exec_stack.len() {
                let v = frame.exec_stack.get(i).unwrap();
                if let Value::Object(id) = v {
                    objs.insert(id);
                }
            }
        }
        objs.into_iter().collect()
    }
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            this: Cell::new(Value::Null),
            arguments: FixedArray32::new(Value::Null),
            locals: FixedArray32::new(Value::Null),
            exec_stack: FixedArray32::new(Value::Null)
        }
    }

    fn reset(&self) {
        self.this.set(Value::Null);
        self.arguments.clear();
        self.locals.clear();
        self.exec_stack.clear();
    }

    pub fn init_with_arguments(&self, this: Value, args: &[Value]) {
        self.this.set(this);
        for arg in args {
            self.arguments.push(*arg);
        }
    }

    #[inline]
    pub fn push_exec(&self, obj: Value) {
        self.exec_stack.push(obj);
    }

    #[inline]
    pub fn pop_exec(&self) -> Value {
        self.exec_stack.pop()
    }

    #[inline]
    pub fn dup_exec(&self) {
        self.exec_stack.push(self.exec_stack.top());
    }

    pub fn map_exec(&self, p: &StackMapPattern, pool: &mut ObjectPool) {
        let mut new_values: SmallVec<[Value; 4]> = SmallVec::with_capacity(p.map.len());
        for loc in &p.map {
            new_values.push(loc.extract(self, pool));
        }

        if p.end_state < 0 {
            for _ in 0..(-p.end_state) {
                self.exec_stack.pop();
            }
        } else {
            for _ in 0..p.end_state {
                self.exec_stack.push(Value::Null);
            }
        }

        for i in 0..new_values.len() {
            let sv_id = self.exec_stack.len() - 1 - i;
            let nv_id = new_values.len() - 1 - i;
            self.exec_stack.set(sv_id, new_values[nv_id]);
        }
    }

    pub fn bulk_load(&self, values: &[Value]) {
        for v in values {
            self.exec_stack.push(*v);
        }
    }

    pub fn reset_locals(&self, n_slots: usize) {
        self.locals.clear();
        for _ in 0..n_slots {
            self.locals.push(Value::Null);
        }
    }

    #[inline]
    pub fn get_local(&self, ind: usize) -> Value {
        self.locals.get(ind).unwrap()
    }

    #[inline]
    pub fn set_local(&self, ind: usize, obj: Value) {
        self.locals.set(ind, obj);
    }

    #[inline]
    pub fn get_argument(&self, id: usize) -> Option<Value> {
        self.arguments.get(id)
    }

    #[inline]
    pub fn must_get_argument(&self, id: usize) -> Value {
        self.get_argument(id).unwrap_or_else(|| {
            panic!(errors::VMError::from(errors::RuntimeError::new("Argument index out of bound")))
        })
    }

    #[inline]
    pub fn get_n_arguments(&self) -> usize {
        self.arguments.len()
    }

    #[inline]
    pub fn get_this(&self) -> Value {
        self.this.get()
    }

    #[inline]
    pub fn set_this(&self, this: Value) {
        self.this.set(this);
    }
}
