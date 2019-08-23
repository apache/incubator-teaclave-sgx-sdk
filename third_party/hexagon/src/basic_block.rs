use std::prelude::v1::*;
use std::collections::HashMap;
use opcode::{OpCode, RtOpCode, StackMapPattern, ValueLocation};
use object_pool::ObjectPool;
use object::Object;
use errors;
use value::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BasicBlock {
    pub(crate) opcodes: Vec<OpCode>
}

impl BasicBlock {
    pub fn from_opcodes(opcodes: Vec<OpCode>) -> BasicBlock {
        BasicBlock {
            opcodes: opcodes
        }
    }

    pub fn join(&mut self, other: BasicBlock) {
        self.opcodes.pop().unwrap();
        for op in other.opcodes {
            self.opcodes.push(op);
        }
    }

    pub fn try_replace_branch_targets(&mut self, to: usize, from: usize) -> bool {
        if self.opcodes.len() == 0 {
            return false;
        }

        let last_opcode_id = self.opcodes.len() - 1;
        let last_opcode = &mut self.opcodes[last_opcode_id];
        match *last_opcode {
            OpCode::ConditionalBranch(if_true, if_false) => {
                if if_true == from || if_false == from {
                    let if_true = if if_true == from {
                        to
                    } else {
                        if_true
                    };
                    let if_false = if if_false == from {
                        to
                    } else {
                        if_false
                    };
                    *last_opcode = OpCode::ConditionalBranch(if_true, if_false);
                    true
                } else {
                    false
                }
            },
            OpCode::Branch(t) => {
                if t == from {
                    *last_opcode = OpCode::Branch(to);
                    true
                } else {
                    false
                }
            },
            _ => false
        }
    }

    pub fn branch_targets(&self) -> (Option<usize>, Option<usize>) {
        if self.opcodes.len() == 0 {
            return (None, None);
        }

        let last_opcode = &self.opcodes[self.opcodes.len() - 1];
        match *last_opcode {
            OpCode::ConditionalBranch(if_true, if_false) => (Some(if_true), Some(if_false)),
            OpCode::Branch(t) => (Some(t), None),
            OpCode::Return => (None, None),
            _ => panic!("Terminator not found")
        }
    }

    pub fn validate(&self, allow_runtime_opcodes: bool) -> Result<(), errors::ValidateError> {
        let mut itr = self.opcodes.iter();
        let mut terminator_found: bool = false;
        let mut stack_depth: isize = 0;

        for op in &mut itr {
            if !allow_runtime_opcodes {
                if let OpCode::Rt(_) = *op {
                    return Err(errors::ValidateError::new("Runtime opcodes are not allowed"));
                }
            }

            op.validate(true)?;

            let (n_pops, n_pushes) = op.get_stack_depth_change();

            stack_depth -= n_pops as isize;

            if stack_depth < 0 {
                return Err(errors::ValidateError::new("Invalid use of stack"));
            }

            stack_depth += n_pushes as isize;

            let terminated = match *op {
                OpCode::ConditionalBranch(_, _)
                    | OpCode::Branch(_)
                    | OpCode::Return => true,
                _ => false
            };
            if terminated {
                terminator_found = true;
                break;
            }
        }

        if stack_depth != 0 {
            return Err(errors::ValidateError::new(format!("Stack not empty at the end of basic block (Depth: {})", stack_depth)));
        }

        if itr.next().is_some() {
            return Err(errors::ValidateError::new("Invalid terminator found in basic block"));
        }

        if !terminator_found {
            return Err(errors::ValidateError::new("Terminator not found"));
        }

        Ok(())
    }

    pub fn build_stack_map(ops: &[BasicStackOp]) -> StackMapPattern {
        if ops.len() == 0 {
            return StackMapPattern {
                map: (&[] as &[ValueLocation]).into(),
                end_state: 0
            };
        }

        let mut lower_bound: isize = 0;
        let mut upper_bound: isize = 0;
        let mut current: isize = 0;

        for op in ops {
            match *op {
                BasicStackOp::Dup => {
                    current += 1;
                },
                BasicStackOp::Pop => {
                    current -= 1;
                },
                BasicStackOp::Rotate2 => {
                    if current - 1 < lower_bound {
                        lower_bound = current - 1;
                    }
                },
                BasicStackOp::Rotate3 => {
                    if current - 2 < lower_bound {
                        lower_bound = current - 2;
                    }
                },
                BasicStackOp::RotateReverse(n) => {
                    let end_id = current - (n as isize - 1);
                    if end_id < lower_bound {
                        lower_bound = end_id;
                    }
                },
                BasicStackOp::LoadInt(_)
                    | BasicStackOp::LoadFloat(_)
                    | BasicStackOp::LoadBool(_)
                    | BasicStackOp::LoadString(_)
                    | BasicStackOp::LoadNull
                    | BasicStackOp::GetLocal(_)
                    | BasicStackOp::GetArgument(_)
                    | BasicStackOp::LoadObject(_)
                    | BasicStackOp::LoadThis => {
                    current += 1;
                }
            }
            if current > upper_bound {
                upper_bound = current;
            }
            if current < lower_bound {
                lower_bound = current;
            }
        }
        let end_state = current;

        let mut stack_map: Vec<ValueLocation> = Vec::new();
        for i in lower_bound..upper_bound + 1 {
            stack_map.push(ValueLocation::Stack(i));
        }

        let mut current: usize = (0 - lower_bound) as usize;
        assert!(stack_map[current] == ValueLocation::Stack(0));

        for op in ops {
            match *op {
                BasicStackOp::Dup => {
                    current += 1;
                    stack_map[current] = stack_map[current - 1].clone();
                },
                BasicStackOp::Pop => {
                    current -= 1;
                },
                BasicStackOp::Rotate2 => {
                    stack_map.swap(current - 1, current);
                },
                BasicStackOp::Rotate3 => {
                    let a = stack_map[current].clone();
                    let b = stack_map[current - 1].clone();
                    let c = stack_map[current - 2].clone();

                    stack_map[current - 2] = b;
                    stack_map[current - 1] = a;
                    stack_map[current] = c;
                },
                BasicStackOp::RotateReverse(n) => {
                    let mut seq: Vec<ValueLocation> = (0..n).map(|i| stack_map[current - i].clone()).collect();
                    for i in 0..n {
                        stack_map[current - i] = seq.pop().unwrap();
                    }
                },
                BasicStackOp::GetLocal(id) => {
                    current += 1;
                    stack_map[current] = ValueLocation::Local(id);
                },
                BasicStackOp::LoadString(ref s) => {
                    current += 1;
                    stack_map[current] = ValueLocation::ConstString(s.clone());
                },
                BasicStackOp::LoadInt(v) => {
                    current += 1;
                    stack_map[current] = ValueLocation::ConstInt(v);
                },
                BasicStackOp::LoadFloat(v) => {
                    current += 1;
                    stack_map[current] = ValueLocation::ConstFloat(v);
                },
                BasicStackOp::LoadBool(v) => {
                    current += 1;
                    stack_map[current] = ValueLocation::ConstBool(v);
                },
                BasicStackOp::LoadNull => {
                    current += 1;
                    stack_map[current] = ValueLocation::ConstNull;
                },
                BasicStackOp::GetArgument(id) => {
                    current += 1;
                    stack_map[current] = ValueLocation::Argument(id);
                },
                BasicStackOp::LoadObject(id) => {
                    current += 1;
                    stack_map[current] = ValueLocation::ConstObject(id);
                },
                BasicStackOp::LoadThis => {
                    current += 1;
                    stack_map[current] = ValueLocation::This;
                }
            }
        }

        let mut begin: usize = 0;
        while begin < (end_state - lower_bound + 1) as usize {
            if stack_map[begin] != ValueLocation::Stack(begin as isize + lower_bound) {
                break;
            }
            begin += 1;
        }

        StackMapPattern {
            map: (begin..(end_state - lower_bound + 1) as usize).map(|i| stack_map[i].clone()).collect(),
            end_state: end_state
        }
    }

    pub fn transform_const_block_locals(&mut self) {
        let mut locals: HashMap<usize, Value> = HashMap::new();
        for i in 1..self.opcodes.len() {
            if let OpCode::GetLocal(id) = self.opcodes[i] {
                if let Some(v) = locals.get(&id) {
                    println!("debug [transform_const_block_locals] Replacing GetLocal({}) with const value {:?}", id, v);
                    self.opcodes[i] = v.to_opcode();
                }
                continue;
            }

            let local_id = match self.opcodes[i] {
                OpCode::SetLocal(id) => id,
                _ => {
                    continue;
                }
            };

            if let Some(_) = locals.remove(&local_id) {
                println!("debug [transform_const_block_locals] Lifetime of const value for local {} ends", local_id);
            }

            let stack_ops: Vec<BasicStackOp> = {
                let mut v: Vec<BasicStackOp> = Vec::new();
                let mut i = (i - 1) as isize;
                while i >= 0 {
                    if let Some(op) = BasicStackOp::from_opcode(&self.opcodes[i as usize]) {
                        v.push(op);
                    } else {
                        break;
                    }
                    i -= 1;
                }
                v.reverse();
                v
            };
            let stack_map = BasicBlock::build_stack_map(stack_ops.as_slice());

            if stack_map.map.len() >= 1 {
                if let Some(v) = stack_map.map[stack_map.map.len() - 1].to_value() {
                    locals.insert(local_id, v);
                    println!("debug [transform_const_block_locals] Lifetime of const value for local {} begins", local_id);
                }
            }
        }
    }

    pub fn transform_const_calls(&mut self) {
        for i in 2..self.opcodes.len() {
            if let OpCode::Call(n_args) = self.opcodes[i] {
                if let Some(target_loc) = ValueLocation::from_opcode(&self.opcodes[i - 1]) {
                    if let Some(this_loc) = ValueLocation::from_opcode(&self.opcodes[i - 2]) {
                        self.opcodes[i - 2] = OpCode::Nop;
                        self.opcodes[i - 1] = OpCode::Nop;
                        self.opcodes[i] = OpCode::Rt(RtOpCode::ConstCall(target_loc, this_loc, n_args));
                    }
                }
            }
        }
    }

    pub fn remove_nops(&mut self) {
        self.opcodes.retain(|v| *v != OpCode::Nop);
    }

    pub fn flatten_stack_maps(&mut self) {
        let mut new_opcodes: Vec<OpCode> = Vec::new();
        for op in &self.opcodes {
            if let OpCode::Rt(RtOpCode::StackMap(ref p)) = *op {
                if let Some(seq) = p.to_opcode_sequence() {
                    println!("debug [flatten_stack_maps] {:?} -> {:?}", p, seq);
                    for v in seq {
                        new_opcodes.push(v);
                    }
                } else {
                    println!("debug [flatten_stack_maps] Cannot convert stack map to opcode sequence");
                    new_opcodes.push(op.clone());
                }
            } else {
                new_opcodes.push(op.clone());
            }
        }
        self.opcodes = new_opcodes;
    }

    pub fn transform_const_get_fields(&mut self, rt_handles: &mut Vec<usize>, pool: &mut ObjectPool, this: Option<Value>) -> bool {
        fn const_get_field_to_opcode(obj: &dyn Object, key: &str, pool: &ObjectPool, rt_handles: &mut Vec<usize>) -> Option<OpCode> {
            if obj.has_const_field(pool, key) {
                if let Some(v) = obj.get_field(pool, key) {
                    println!("debug [transform_const_get_fields] GetField/CallField {} -> {:?}", key, v);
                    if let Value::Object(id) = v {
                        rt_handles.push(id);
                    }
                    Some(v.to_opcode())
                } else {
                    println!("debug [transform_const_get_fields] Field {} is marked as const but has no value", key);
                    None
                }
            } else {
                println!("debug [transform_const_get_fields] Field {} is not const", key);
                None
            }
        }
        fn extract_object_id(loc: &ValueLocation, this: &Option<Value>) -> Option<usize> {
            if let ValueLocation::ConstObject(id) = *loc {
                Some(id)
            } else if *loc == ValueLocation::This && this.is_some() {
                if let Value::Object(id) = this.unwrap() {
                    Some(id)
                } else {
                    None
                }
            } else {
                None
            }
        }

        let mut optimized: bool = false;

        for i in 2..self.opcodes.len() {
            if let Some(_) = BasicStackOp::from_opcode(&self.opcodes[i]) {
                continue;
            }

            // FIXME: Remove this loop and the expensive build_stack_map calls
            // Maybe iterators ?
            let stack_ops: Vec<BasicStackOp> = {
                let mut v: Vec<BasicStackOp> = Vec::new();
                let mut i = (i - 1) as isize;
                while i >= 0 {
                    if let Some(op) = BasicStackOp::from_opcode(&self.opcodes[i as usize]) {
                        v.push(op);
                    } else {
                        break;
                    }
                    i -= 1;
                }
                v.reverse();
                v
            };
            let stack_map = BasicBlock::build_stack_map(stack_ops.as_slice());

            //println!("debug [transform_const_get_fields] stack_map: {:?}", stack_map);

            if stack_map.map.len() >= 2 {
                match self.opcodes[i] {
                    OpCode::GetField => {
                        let obj_id = extract_object_id(&stack_map.map[stack_map.map.len() - 1], &this);
                        if let Some(obj_id) = obj_id {
                            if let ValueLocation::ConstObject(key_id) = stack_map.map[stack_map.map.len() - 2] {
                                let obj = pool.get_direct(obj_id);
                                let key = pool.get_direct(key_id).to_string();
                                let mut target_opcode: Option<OpCode> = const_get_field_to_opcode(
                                    obj,
                                    key.as_str(),
                                    pool,
                                    rt_handles
                                );

                                if target_opcode.is_none() {
                                    target_opcode = Some(OpCode::Rt(RtOpCode::ConstGetField(
                                        obj_id,
                                        Value::Object(key_id)
                                    )));
                                }

                                if let Some(op) = target_opcode {
                                    for j in 1..stack_ops.len() + 1 {
                                        self.opcodes[i - j] = OpCode::Nop;
                                    }
                                    let mut stack_map = stack_map.clone();
                                    stack_map.map.pop().unwrap();
                                    stack_map.map.pop().unwrap();
                                    stack_map.end_state -= 2;
                                    self.opcodes[i - 1] = OpCode::Rt(RtOpCode::StackMap(stack_map));
                                    self.opcodes[i] = op;
                                    optimized = true;
                                }
                            }
                        }
                    },
                    OpCode::CallField(n_args) if stack_map.map.len() >= 3 => {
                        let obj_id = extract_object_id(&stack_map.map[stack_map.map.len() - 1], &this);
                        if let Some(obj_id) = obj_id {
                            // opcodes[i - 2] is the target `this` object
                            if let ValueLocation::ConstObject(key_id) = stack_map.map[stack_map.map.len() - 3] {
                                let obj = pool.get_direct(obj_id);
                                let key = pool.get_direct(key_id).to_string();
                                let target_opcode: Option<OpCode> = const_get_field_to_opcode(
                                    obj,
                                    key.as_str(),
                                    pool,
                                    rt_handles
                                );

                                if let Some(op) = target_opcode {
                                    // Original layout: key, this, target, call_field
                                    // New layout: this, target, call
                                    for j in 1..stack_ops.len() + 1 {
                                        self.opcodes[i - j] = OpCode::Nop;
                                    }
                                    let mut stack_map = stack_map.clone();

                                    stack_map.map.pop().unwrap();
                                    let b = stack_map.map.pop().unwrap(); // this
                                    stack_map.map.pop().unwrap();
                                    stack_map.map.push(b);
                                    stack_map.end_state -= 2;

                                    self.opcodes[i - 2] = OpCode::Rt(RtOpCode::StackMap(stack_map));
                                    self.opcodes[i - 1] = op;
                                    self.opcodes[i] = OpCode::Call(n_args);
                                    optimized = true;
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }

        optimized
    }

    pub fn transform_const_string_loads(&mut self, rt_handles: &mut Vec<usize>, pool: &mut ObjectPool) {
        for op in &mut self.opcodes {
            if let OpCode::LoadString(ref s) = *op {
                let s = s.clone();
                let obj_id = pool.allocate(Box::new(s));
                rt_handles.push(obj_id);
                *op = OpCode::Rt(RtOpCode::LoadObject(obj_id));
            }
        }
    }

    pub fn transform_const_static_loads(&mut self, _rt_handles: &mut Vec<usize>, pool: &mut ObjectPool) {
        for i in 1..self.opcodes.len() {
            if self.opcodes[i] == OpCode::GetStatic {
                // We assume the LoadString -> LoadObject trans. is already done.
                if let OpCode::Rt(RtOpCode::LoadObject(key_id)) = self.opcodes[i - 1] {
                    let key = pool.get_direct(key_id).to_string();
                    if let Some(v) = pool.get_static_object(key.as_str()) {
                        self.opcodes[i - 1] = OpCode::Nop;
                        self.opcodes[i] = v.to_opcode();
                    }
                }
            }
        }
    }

    pub fn build_bulk_loads(&mut self) {
        fn build(values: Vec<Value>, target: &mut Vec<OpCode>) {
            if !values.is_empty() {
                if values.len() >= 3 {
                    println!("debug [build_bulk_loads] Packing sequence: {:?}", values);
                    target.push(OpCode::Rt(RtOpCode::BulkLoad(values.into())));
                } else {
                    println!("debug [build_bulk_loads] Not packing sequence: {:?}", values);
                    for v in values {
                        target.push(OpCode::from_value(v));
                    }
                }
            }
        }

        let mut deferred_values: Vec<Value> = Vec::new();
        let mut new_opcodes: Vec<OpCode> = Vec::new();

        for op in &self.opcodes {
            if let Some(v) = op.to_value() {
                deferred_values.push(v);
            } else {
                build(
                    ::std::mem::replace(&mut deferred_values, Vec::new()),
                    &mut new_opcodes
                );
                new_opcodes.push(op.clone());
            }
        }
        build(
            ::std::mem::replace(&mut deferred_values, Vec::new()),
            &mut new_opcodes
        );
        self.opcodes = new_opcodes;
    }

    pub fn rebuild_stack_patterns(&mut self) {
        fn pack_deferred_ops(ops: Vec<BasicStackOp>) -> PackResult {
            if ops.len() == 0 {
                return PackResult::Noop;
            }
            if ops.len() <= 2 {
                return PackResult::Restore(ops);
            }

            let pattern = BasicBlock::build_stack_map(ops.as_slice());

            if pattern.map.len() == 0 && pattern.end_state == 0 {
                println!("debug [pack_deferred_ops] No-op detected");
                return PackResult::Noop;
            }

            if pattern.map.len() as f64 > ops.len() as f64 * 0.6 {
                println!("debug [pack_deferred_ops] Result worse than expected. Rolling back.");
                return PackResult::Restore(ops);
            }

            let result = OpCode::Rt(RtOpCode::StackMap(pattern));

            println!("debug [pack_deferred_ops] {:?} -> {:?}", ops, result);
            PackResult::OkWithResult(result)
        }

        let mut new_ops: Vec<OpCode> = Vec::new();
        let mut deferred_stack_ops: Vec<BasicStackOp> = Vec::new();

        for op in &self.opcodes {
            match BasicStackOp::from_opcode(op) {
                Some(v) => deferred_stack_ops.push(v),
                None => {
                    let packed = pack_deferred_ops(::std::mem::replace(&mut deferred_stack_ops, Vec::new()));
                    match packed {
                        PackResult::OkWithResult(v) => {
                            new_ops.push(v);
                        },
                        PackResult::Noop => {},
                        PackResult::Restore(seq) => {
                            for v in seq {
                                new_ops.push(v.to_opcode());
                            }
                        }
                    }
                    new_ops.push(op.clone());
                }
            }
        }

        let packed = pack_deferred_ops(::std::mem::replace(&mut deferred_stack_ops, Vec::new()));
        match packed {
            PackResult::OkWithResult(v) => {
                new_ops.push(v);
            },
            PackResult::Noop => {},
            PackResult::Restore(seq) => {
                for v in seq {
                    new_ops.push(v.to_opcode());
                }
            }
        }

        self.opcodes = new_ops;
    }
}

enum PackResult {
    OkWithResult(OpCode),
    Noop,
    Restore(Vec<BasicStackOp>)
}

#[derive(Clone, Debug)]
pub enum BasicStackOp {
    Dup,
    Pop,
    Rotate2,
    Rotate3,
    RotateReverse(usize),
    GetLocal(usize),
    LoadString(String),
    LoadInt(i64),
    LoadFloat(f64),
    LoadBool(bool),
    LoadNull,
    LoadObject(usize),
    LoadThis,
    GetArgument(usize)
}

impl BasicStackOp {
    pub fn to_opcode(&self) -> OpCode {
        match *self {
            BasicStackOp::Dup => OpCode::Dup,
            BasicStackOp::Pop => OpCode::Pop,
            BasicStackOp::Rotate2 => OpCode::Rotate2,
            BasicStackOp::Rotate3 => OpCode::Rotate3,
            BasicStackOp::RotateReverse(n) => OpCode::RotateReverse(n),
            BasicStackOp::LoadInt(v) => OpCode::LoadInt(v),
            BasicStackOp::LoadFloat(v) => OpCode::LoadFloat(v),
            BasicStackOp::LoadString(ref v) => OpCode::LoadString(v.clone()),
            BasicStackOp::LoadBool(v) => OpCode::LoadBool(v),
            BasicStackOp::LoadNull => OpCode::LoadNull,
            BasicStackOp::GetLocal(id) => OpCode::GetLocal(id),
            BasicStackOp::GetArgument(id) => OpCode::GetArgument(id),
            BasicStackOp::LoadObject(id) => OpCode::Rt(RtOpCode::LoadObject(id)),
            BasicStackOp::LoadThis => OpCode::LoadThis
        }
    }

    pub fn from_opcode(op: &OpCode) -> Option<BasicStackOp> {
        match *op {
            OpCode::Dup => {
                Some(BasicStackOp::Dup)
            },
            OpCode::Pop => {
                Some(BasicStackOp::Pop)
            },
            OpCode::Rotate2 => {
                Some(BasicStackOp::Rotate2)
            },
            OpCode::Rotate3 => {
                Some(BasicStackOp::Rotate3)
            },
            OpCode::RotateReverse(n) => {
                Some(BasicStackOp::RotateReverse(n))
            },
            OpCode::LoadInt(v) => {
                Some(BasicStackOp::LoadInt(v))
            },
            OpCode::LoadFloat(v) => {
                Some(BasicStackOp::LoadFloat(v))
            },
            OpCode::LoadString(ref s) => {
                Some(BasicStackOp::LoadString(s.clone()))
            },
            OpCode::LoadBool(v) => {
                Some(BasicStackOp::LoadBool(v))
            },
            OpCode::LoadNull => {
                Some(BasicStackOp::LoadNull)
            },
            OpCode::GetLocal(id) => {
                Some(BasicStackOp::GetLocal(id))
            },
            OpCode::GetArgument(id) => {
                Some(BasicStackOp::GetArgument(id))
            },
            OpCode::Rt(RtOpCode::LoadObject(id)) => {
                Some(BasicStackOp::LoadObject(id))
            },
            OpCode::LoadThis => {
                Some(BasicStackOp::LoadThis)
            },
            _ => None
        }
    }
}
