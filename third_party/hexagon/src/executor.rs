use std::prelude::v1::*;
use std::cell::{Ref, RefMut, RefCell};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::cmp::Ordering;
use object::Object;
use call_stack::{CallStack, FrameHandle};
use opcode::{OpCode, RtOpCode, SelectType};
use errors;
use basic_block::BasicBlock;
use object_pool::ObjectPool;
use smallvec::SmallVec;
use hybrid::executor::Executor as HybridExecutor;
use value::{Value, ValueContext};
use builtin::BuiltinObject;
use generic_arithmetic;

pub struct Executor {
    inner: RefCell<ExecutorImpl>
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            inner: RefCell::new(ExecutorImpl::new())
        }
    }

    pub fn handle<'a>(&'a self) -> Ref<'a, ExecutorImpl> {
        self.inner.borrow()
    }

    pub fn handle_mut<'a>(&'a self) -> RefMut<'a, ExecutorImpl> {
        self.inner.borrow_mut()
    }
}

pub struct ExecutorImpl {
    stack: CallStack,
    hybrid_executor: HybridExecutor,
    pub log_execution: bool,

    object_pool: ObjectPool
}

enum EvalControlMessage {
    Return(Value),
    Redirect(usize)
}

macro_rules! eval_select_opcode_sequence {
    ($self:ident, $seq:expr) => (for op in $seq {
        if $self._eval_opcode(op).is_some() {
            panic!(errors::VMError::from(
                "Attempting to modify the control flow from inside a Select block"
            ));
        }
    })
}

impl ExecutorImpl {
    pub fn new() -> ExecutorImpl {
        let mut ret = ExecutorImpl {
            stack: CallStack::new(2048),
            hybrid_executor: HybridExecutor::new(),
            log_execution: false,
            object_pool: ObjectPool::new()
        };
        ret.create_static_object("__builtin", Box::new(BuiltinObject::new()));
        ret
    }

    #[inline]
    pub fn get_current_frame<'a>(&self) -> &FrameHandle {
        self.stack.top()
    }

    #[inline]
    pub fn get_object_pool(&self) -> &ObjectPool {
        &self.object_pool
    }

    #[inline]
    pub fn get_object_pool_mut(&mut self) -> &mut ObjectPool {
        &mut self.object_pool
    }

    pub fn set_stack_limit(&mut self, limit: usize) {
        self.stack.set_limit(limit);
    }

    pub fn get_hybrid_executor(&self) -> &HybridExecutor {
        &self.hybrid_executor
    }

    pub fn get_hybrid_executor_mut(&mut self) -> &mut HybridExecutor {
        &mut self.hybrid_executor
    }

    pub fn invoke(&mut self, callable_val: Value, this: Value, field_name: Option<&str>, args: &[Value]) {
        // Push the callable object onto the execution stack
        // to prevent it from begin GC-ed.
        //
        // No extra care needs to be taken for arguments
        // bacause they are already on the new frame.

        let callable_obj_id = match callable_val {
            Value::Object(id) => id,
            _ => panic!(errors::VMError::from(
                format!("Not callable. Got: {:?}", callable_val)
            ))
        };

        let callable_obj = self.object_pool.get(callable_obj_id);

        self.get_current_frame().push_exec(callable_val);
        self.stack.push();

        let ret = catch_unwind(AssertUnwindSafe(|| {
            let frame = self.stack.top();
            frame.init_with_arguments(match this {
                Value::Null => self.get_current_frame().get_this(),
                _ => this
            }, args);
            match field_name {
                Some(v) => callable_obj.call_field(v, self),
                None => callable_obj.call(self)
            }
        }));

        self.stack.pop();
        self.get_current_frame().pop_exec();

        match ret {
            Ok(v) => {
                self.get_current_frame().push_exec(v);
            },
            Err(e) => resume_unwind(e)
        }
    }

    fn set_static_object<K: ToString>(&mut self, key: K, obj: Value) {
        self.get_object_pool_mut().set_static_object(key, obj);
    }

    pub fn create_static_object<K: ToString>(&mut self, key: K, obj: Box<Object>) {
        let obj_id = self.object_pool.allocate(obj);
        self.set_static_object(key, Value::Object(obj_id));
    }

    pub fn get_static_object<K: AsRef<str>>(&self, key: K) -> Option<&Value> {
        self.get_object_pool().get_static_object(key)
    }

    fn _call_impl(&mut self, n_args: usize) {
        let (target, this, args) = {
            let frame = self.get_current_frame();

            let target = frame.pop_exec();
            let this = frame.pop_exec();

            let mut args: SmallVec<[Value; 4]> = SmallVec::with_capacity(n_args);
            for _ in 0..n_args {
                args.push(frame.pop_exec());
            }

            (target, this, args)
        };
        self.invoke(target, this, None, args.as_slice());
    }

    fn _call_field_impl(&mut self, n_args: usize) {
        let (target, this, field_name, args) = {
            let frame = self.get_current_frame();

            let target = frame.pop_exec();
            let this = frame.pop_exec();
            let field_name = frame.pop_exec();

            let mut args: SmallVec<[Value; 4]> = SmallVec::with_capacity(n_args);
            for _ in 0..n_args {
                args.push(frame.pop_exec());
            }

            (target, this, field_name, args)
        };
        let field_name = ValueContext::new(&field_name, self.get_object_pool()).to_str().to_string();
        self.invoke(target, this, Some(field_name.as_str()), args.as_slice());
    }

    fn _get_field_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let target_obj_val = frame.pop_exec();
        let target_obj = ValueContext::new(
            &target_obj_val,
            pool
        ).as_object_direct();

        let key_val = frame.pop_exec();
        let key = ValueContext::new(
            &key_val,
            pool
        ).as_object_direct().to_str();

        if let Some(v) = target_obj.get_field(pool, key) {
            frame.push_exec(v);
        } else {
            frame.push_exec(Value::Null);
        }
    }

    fn _set_field_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (target_obj_val, key_val, value) = (
            frame.pop_exec(),
            frame.pop_exec(),
            frame.pop_exec()
        );

        let target_obj = ValueContext::new(
            &target_obj_val,
            pool
        ).as_object_direct();

        let key = ValueContext::new(
            &key_val,
            pool
        ).as_object_direct().to_str();

        target_obj.set_field(key, value);
    }

    fn _int_add_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            (
                ValueContext::new(&left, pool).to_i64(),
                ValueContext::new(&right, pool).to_i64(),
            )
        };

        frame.push_exec(Value::Int(left + right));
    }

    fn _int_sub_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            (
                ValueContext::new(&left, pool).to_i64(),
                ValueContext::new(&right, pool).to_i64(),
            )
        };

        frame.push_exec(Value::Int(left - right));
    }

    fn _int_mul_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            (
                ValueContext::new(&left, pool).to_i64(),
                ValueContext::new(&right, pool).to_i64(),
            )
        };

        frame.push_exec(Value::Int(left * right));
    }

    fn _int_div_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            (
                ValueContext::new(&left, pool).to_i64(),
                ValueContext::new(&right, pool).to_i64(),
            )
        };

        frame.push_exec(Value::Int(left / right));
    }

    fn _int_mod_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            (
                ValueContext::new(&left, pool).to_i64(),
                ValueContext::new(&right, pool).to_i64(),
            )
        };

        frame.push_exec(Value::Int(left % right));
    }

    fn _int_pow_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            (
                ValueContext::new(&left, pool).to_i64(),
                ValueContext::new(&right, pool).to_i64(),
            )
        };

        frame.push_exec(Value::Int(left.pow(right as u32)));
    }

    fn _float_add_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            (
                ValueContext::new(&left, pool).to_f64(),
                ValueContext::new(&right, pool).to_f64(),
            )
        };

        frame.push_exec(Value::Float(left + right));
    }

    fn _float_sub_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            (
                ValueContext::new(&left, pool).to_f64(),
                ValueContext::new(&right, pool).to_f64(),
            )
        };

        frame.push_exec(Value::Float(left - right));
    }

    fn _float_mul_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            (
                ValueContext::new(&left, pool).to_f64(),
                ValueContext::new(&right, pool).to_f64(),
            )
        };

        frame.push_exec(Value::Float(left * right));
    }

    fn _float_div_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            (
                ValueContext::new(&left, pool).to_f64(),
                ValueContext::new(&right, pool).to_f64(),
            )
        };

        frame.push_exec(Value::Float(left / right));
    }

    fn _float_mod_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            (
                ValueContext::new(&left, pool).to_f64(),
                ValueContext::new(&right, pool).to_f64(),
            )
        };

        frame.push_exec(Value::Float(left % right));
    }

    fn _float_powi_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            (
                ValueContext::new(&left, pool).to_f64(),
                ValueContext::new(&right, pool).to_i64(),
            )
        };

        frame.push_exec(Value::Float(left.powi(right as i32)));
    }

    fn _float_powf_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            (
                ValueContext::new(&left, pool).to_f64(),
                ValueContext::new(&right, pool).to_f64(),
            )
        };

        frame.push_exec(Value::Float(left.powf(right)));
    }

    fn _string_add_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &mut self.object_pool;

        let new_value = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());
            let (left, right) = (
                ValueContext::new(&left, pool),
                ValueContext::new(&right, pool)
            );
            format!("{}{}", left.to_str(), right.to_str())
        };
        let new_value = pool.allocate(
            Box::new(new_value)
        );

        frame.push_exec(Value::Object(
            new_value
        ));
    }

    fn _cast_to_float_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let value = ValueContext::new(
            &frame.pop_exec(),
            pool
        ).to_f64();
        frame.push_exec(Value::Float(value));
    }

    fn _cast_to_int_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let value = ValueContext::new(
            &frame.pop_exec(),
            pool
        ).to_i64();
        frame.push_exec(Value::Int(value));
    }

    fn _cast_to_bool_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let value = ValueContext::new(
            &frame.pop_exec(),
            pool
        ).to_bool();
        frame.push_exec(Value::Bool(value));
    }

    fn _cast_to_string_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &mut self.object_pool;

        let value = ValueContext::new(
            &frame.pop_exec(),
            pool
        ).to_str().to_string();
        let value = pool.allocate(
            Box::new(value)
        );
        frame.push_exec(Value::Object(value));
    }

    fn _and_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = (frame.pop_exec(), frame.pop_exec());

        let left_ctx = ValueContext::new(
            &left,
            pool
        );
        let right_ctx = ValueContext::new(
            &right,
            pool
        );

        frame.push_exec(Value::Bool(left_ctx.to_bool() && right_ctx.to_bool()));
    }

    fn _or_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let (left, right) = (frame.pop_exec(), frame.pop_exec());

        let left_ctx = ValueContext::new(
            &left,
            pool
        );
        let right_ctx = ValueContext::new(
            &right,
            pool
        );

        frame.push_exec(Value::Bool(left_ctx.to_bool() || right_ctx.to_bool()));
    }

    fn _not_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let value = ValueContext::new(
            &frame.pop_exec(),
            pool
        ).to_bool();
        frame.push_exec(Value::Bool(!value));
    }

    fn _test_lt_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let ord = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());

            let left_ctx = ValueContext::new(
                &left,
                pool
            );
            let right_ctx = ValueContext::new(
                &right,
                pool
            );
            left_ctx.compare(&right_ctx)
        };

        frame.push_exec(Value::Bool(ord == Some(Ordering::Less)));
    }

    fn _test_le_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let ord = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());

            let left_ctx = ValueContext::new(
                &left,
                pool
            );
            let right_ctx = ValueContext::new(
                &right,
                pool
            );
            left_ctx.compare(&right_ctx)
        };

        frame.push_exec(Value::Bool(
            ord == Some(Ordering::Less) || ord == Some(Ordering::Equal)
        ));
    }

    fn _test_eq_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let ord = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());

            let left_ctx = ValueContext::new(
                &left,
                pool
            );
            let right_ctx = ValueContext::new(
                &right,
                pool
            );
            left_ctx.compare(&right_ctx)
        };

        frame.push_exec(Value::Bool(ord == Some(Ordering::Equal)));
    }

    fn _test_ne_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let ord = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());

            let left_ctx = ValueContext::new(
                &left,
                pool
            );
            let right_ctx = ValueContext::new(
                &right,
                pool
            );
            left_ctx.compare(&right_ctx)
        };

        frame.push_exec(Value::Bool(ord != Some(Ordering::Equal)));
    }

    fn _test_ge_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let ord = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());

            let left_ctx = ValueContext::new(
                &left,
                pool
            );
            let right_ctx = ValueContext::new(
                &right,
                pool
            );
            left_ctx.compare(&right_ctx)
        };

        frame.push_exec(Value::Bool(
            ord == Some(Ordering::Greater) || ord == Some(Ordering::Equal)
        ));
    }

    fn _test_gt_impl(&mut self) {
        let frame = self.stack.top();
        let pool = &self.object_pool;

        let ord = {
            let (left, right) = (frame.pop_exec(), frame.pop_exec());

            let left_ctx = ValueContext::new(
                &left,
                pool
            );
            let right_ctx = ValueContext::new(
                &right,
                pool
            );
            left_ctx.compare(&right_ctx)
        };

        frame.push_exec(Value::Bool(ord == Some(Ordering::Greater)));
    }

    fn _rotate2_impl(&mut self) {
        let frame = self.get_current_frame();
        let a = frame.pop_exec();
        let b = frame.pop_exec();

        frame.push_exec(a);
        frame.push_exec(b);
    }

    fn _rotate3_impl(&mut self) {
        let frame = self.get_current_frame();
        let a = frame.pop_exec();
        let b = frame.pop_exec();
        let c = frame.pop_exec();

        frame.push_exec(b);
        frame.push_exec(a);
        frame.push_exec(c);
    }

    fn _rotate_reverse_impl(&mut self, n: usize) {
        let frame = self.get_current_frame();
        if n <= 4 {
            let mut t = [Value::Null; 4];
            for i in 0..n {
                t[i] = frame.pop_exec();
            }
            for i in 0..n {
                frame.push_exec(t[i]);
            }
        } else {
            let mut t = Vec::with_capacity(n);
            for _ in 0..n {
                t.push(frame.pop_exec());
            }
            for i in 0..n {
                frame.push_exec(t[i]);
            }
        }
    }

    fn _rt_dispatch_impl(&mut self, op: &RtOpCode) {
        match *op {
            RtOpCode::LoadObject(id) => {
                self.get_current_frame().push_exec(Value::Object(id));
            },
            RtOpCode::BulkLoad(ref values) => {
                self.get_current_frame().bulk_load(values.as_slice());
            },
            RtOpCode::StackMap(ref map) => {
                let frame = self.stack.top();
                let pool = &mut self.object_pool;
                frame.map_exec(map, pool);
            },
            RtOpCode::ConstCall(ref target, ref this, n_args) => {
                let frame = self.stack.top();
                let pool = &mut self.object_pool;

                let target = target.extract(&*frame, pool);
                let this = this.extract(&*frame, pool);

                let mut args: SmallVec<[Value; 4]> = SmallVec::with_capacity(n_args);
                for _ in 0..n_args {
                    args.push(frame.pop_exec());
                }

                self.invoke(target, this, None, args.as_slice());
            },
            RtOpCode::ConstGetField(target, key) => {
                let frame = self.stack.top();
                let pool = &self.object_pool;

                let target_obj = pool.get_direct(target);
                let key = ValueContext::new(
                    &key,
                    pool
                ).as_object_direct().to_str();

                if let Some(v) = target_obj.get_field(pool, key) {
                    frame.push_exec(v);
                } else {
                    frame.push_exec(Value::Null);
                }
            }
        }
    }

    fn _eval_opcode(&mut self, op: &OpCode) -> Option<EvalControlMessage> {
        if self.log_execution {
            println!("[_eval_opcode] {:?}", op);
        }

        match *op {
            OpCode::Nop => {},
            OpCode::LoadNull => {
                self.get_current_frame().push_exec(Value::Null);
            },
            OpCode::LoadInt(value) => {
                self.get_current_frame().push_exec(Value::Int(value));
            },
            OpCode::LoadFloat(value) => {
                self.get_current_frame().push_exec(Value::Float(value));
            },
            OpCode::LoadBool(value) => {
                self.get_current_frame().push_exec(Value::Bool(value));
            },
            OpCode::LoadString(ref value) => {
                let obj = self.object_pool.allocate(Box::new(value.clone()));
                self.get_current_frame().push_exec(Value::Object(obj));
            },
            OpCode::LoadThis => {
                let frame = self.get_current_frame();
                frame.push_exec(frame.get_this());
            },
            OpCode::Call(n_args) => {
                self._call_impl(n_args);
            },
            OpCode::CallField(n_args) => {
                self._call_field_impl(n_args);
            },
            OpCode::Pop => {
                self.get_current_frame().pop_exec();
            },
            OpCode::Dup => {
                self.get_current_frame().dup_exec();
            },
            OpCode::InitLocal(n_slots) => {
                let frame = self.get_current_frame();
                frame.reset_locals(n_slots);
            },
            OpCode::GetLocal(ind) => {
                let frame = self.get_current_frame();
                let ret = frame.get_local(ind);
                frame.push_exec(ret);
            },
            OpCode::SetLocal(ind) => {
                let frame = self.get_current_frame();
                let value = frame.pop_exec();
                frame.set_local(ind, value);
            },
            OpCode::GetArgument(ind) => {
                let frame = self.get_current_frame();
                frame.push_exec(frame.must_get_argument(ind));
            },
            OpCode::GetNArguments => {
                let frame = self.get_current_frame();
                frame.push_exec(Value::Int(frame.get_n_arguments() as i64));
            },
            OpCode::GetStatic => {
                let frame = self.stack.top();
                let pool = &self.object_pool;

                let key_val = frame.pop_exec();
                let key = ValueContext::new(
                    &key_val,
                    pool
                ).as_object_direct().to_str();
                let maybe_target_obj = self.get_static_object(key).map(|v| *v);

                if let Some(target_obj) = maybe_target_obj {
                    frame.push_exec(target_obj);
                } else {
                    frame.push_exec(Value::Null);
                }
            },
            OpCode::SetStatic => {
                let frame = self.stack.top();
                let pool = &self.object_pool;

                let key_val = frame.pop_exec();
                let key = ValueContext::new(
                    &key_val,
                    pool
                ).as_object_direct().to_string();

                let value = frame.pop_exec();

                self.set_static_object(key, value);
            },
            OpCode::GetField => {
                self._get_field_impl();
            },
            OpCode::SetField => {
                self._set_field_impl();
            },
            OpCode::Branch(target_id) => {
                return Some(EvalControlMessage::Redirect(target_id));
            },
            OpCode::ConditionalBranch(if_true, if_false) => {
                let condition_is_true = {
                    let frame = self.get_current_frame();
                    ValueContext::new(
                        &frame.pop_exec(),
                        self.get_object_pool()
                    ).to_bool()
                };

                return Some(EvalControlMessage::Redirect(if condition_is_true {
                    if_true
                } else {
                    if_false
                }));
            },
            OpCode::Return => {
                let ret_val = self.get_current_frame().pop_exec();
                return Some(EvalControlMessage::Return(ret_val));
            },
            OpCode::Add => {
                let (left, right) = (self.get_current_frame().pop_exec(), self.get_current_frame().pop_exec());
                let ret = generic_arithmetic::exec_add(self, left, right);
                self.get_current_frame().push_exec(ret);
            },
            OpCode::Sub => {
                let (left, right) = (self.get_current_frame().pop_exec(), self.get_current_frame().pop_exec());
                let ret = generic_arithmetic::exec_sub(self, left, right);
                self.get_current_frame().push_exec(ret);
            },
            OpCode::Mul => {
                let (left, right) = (self.get_current_frame().pop_exec(), self.get_current_frame().pop_exec());
                let ret = generic_arithmetic::exec_mul(self, left, right);
                self.get_current_frame().push_exec(ret);
            },
            OpCode::Div => {
                let (left, right) = (self.get_current_frame().pop_exec(), self.get_current_frame().pop_exec());
                let ret = generic_arithmetic::exec_div(self, left, right);
                self.get_current_frame().push_exec(ret);
            },
            OpCode::Mod => {
                let (left, right) = (self.get_current_frame().pop_exec(), self.get_current_frame().pop_exec());
                let ret = generic_arithmetic::exec_mod(self, left, right);
                self.get_current_frame().push_exec(ret);
            },
            OpCode::Pow => {
                let (left, right) = (self.get_current_frame().pop_exec(), self.get_current_frame().pop_exec());
                let ret = generic_arithmetic::exec_pow(self, left, right);
                self.get_current_frame().push_exec(ret);
            },
            OpCode::IntAdd => {
                self._int_add_impl();
            },
            OpCode::IntSub => {
                self._int_sub_impl();
            },
            OpCode::IntMul => {
                self._int_mul_impl();
            },
            OpCode::IntDiv => {
                self._int_div_impl();
            },
            OpCode::IntMod => {
                self._int_mod_impl();
            },
            OpCode::IntPow => {
                self._int_pow_impl();
            },
            OpCode::FloatAdd => {
                self._float_add_impl();
            },
            OpCode::FloatSub => {
                self._float_sub_impl();
            },
            OpCode::FloatMul => {
                self._float_mul_impl();
            },
            OpCode::FloatDiv => {
                self._float_div_impl();
            },
            OpCode::FloatPowi => {
                self._float_powi_impl();
            },
            OpCode::FloatPowf => {
                self._float_powf_impl();
            },
            OpCode::StringAdd => {
                self._string_add_impl();
            },
            OpCode::CastToFloat => {
                self._cast_to_float_impl();
            },
            OpCode::CastToInt => {
                self._cast_to_int_impl();
            },
            OpCode::CastToBool => {
                self._cast_to_bool_impl();
            },
            OpCode::CastToString => {
                self._cast_to_string_impl();
            },
            OpCode::And => {
                self._and_impl();
            },
            OpCode::Or => {
                self._or_impl();
            },
            OpCode::Not => {
                self._not_impl();
            },
            OpCode::TestLt => {
                self._test_lt_impl();
            },
            OpCode::TestLe => {
                self._test_le_impl();
            },
            OpCode::TestEq => {
                self._test_eq_impl();
            },
            OpCode::TestNe => {
                self._test_ne_impl();
            },
            OpCode::TestGe => {
                self._test_ge_impl();
            },
            OpCode::TestGt => {
                self._test_gt_impl();
            },
            OpCode::Rotate2 => {
                self._rotate2_impl();
            },
            OpCode::Rotate3 => {
                self._rotate3_impl();
            },
            OpCode::RotateReverse(n) => {
                self._rotate_reverse_impl(n);
            },
            OpCode::Rt(ref op) => {
                self._rt_dispatch_impl(op);
            },
            OpCode::Select(ref t, ref left, ref right) => {
                eval_select_opcode_sequence!(self, left);
                let left_val = ValueContext::new(
                    &self.stack.top().pop_exec(),
                    &self.object_pool
                ).to_bool();

                let result = match *t {
                    SelectType::And => {
                        if left_val {
                            eval_select_opcode_sequence!(self, right);
                            let v = ValueContext::new(
                                &self.stack.top().pop_exec(),
                                &self.object_pool
                            ).to_bool();
                            if v {
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    },
                    SelectType::Or => {
                        if !left_val {
                            eval_select_opcode_sequence!(self, right);
                            let v = ValueContext::new(
                                &self.stack.top().pop_exec(),
                                &self.object_pool
                            ).to_bool();
                            if !v {
                                false
                            } else {
                                true
                            }
                        } else {
                            true
                        }
                    }
                };
                self.stack.top().push_exec(Value::Bool(result));
            }
        }

        None
    }

    fn eval_basic_blocks_impl(&mut self, bb: &BasicBlock) -> EvalControlMessage {
        if self.object_pool.get_alloc_count() >= 1000 {
            self.object_pool.reset_alloc_count();
            self.object_pool.collect(&self.stack);
        }

        for op in &bb.opcodes {
            if let Some(msg) = self._eval_opcode(op) {
                return msg;
            }
        }

        panic!(errors::VMError::from(errors::RuntimeError::new("Leaving a basic block without terminator")));
    }

    pub(crate) fn eval_basic_blocks(&mut self, basic_blocks: &[BasicBlock], basic_block_id: usize) -> Value {
        let mut current_id = basic_block_id;

        loop {
            let msg = self.eval_basic_blocks_impl(&basic_blocks[current_id]);
            match msg {
                EvalControlMessage::Redirect(target) => {
                    current_id = target;
                },
                EvalControlMessage::Return(value) => {
                    return value;
                }
            }
        }
    }

    pub fn gc(&mut self) {
        self.object_pool.collect(&self.stack);
    }

    pub fn run_callable<K: AsRef<str>>(&mut self, key: K) -> Result<(), errors::VMError> {
        let callable_obj_id = *self.get_static_object(key).unwrap_or_else(|| {
            panic!(errors::VMError::from(errors::RuntimeError::new("Static object not found")));
        });

        let new_this = Value::Null;

        self.stack.push();
        let ret = catch_unwind(AssertUnwindSafe(|| {
            let frame = self.stack.top();
            frame.init_with_arguments(
                new_this,
                &[]
            );
            self.invoke(callable_obj_id, new_this, None, &[])
        }));
        self.stack.pop();

        match ret {
            Ok(_) => Ok(()),
            Err(e) => {
                if let Ok(e) = e.downcast::<errors::VMError>() {
                    Err(*e)
                } else {
                    panic!("Unknown error from VM");
                }
            }
        }
    }
}
