use std::prelude::v1::*;
use std::any::Any;
use std::cell::RefCell;
use object::Object;
use object_pool::ObjectPool;
use basic_block::BasicBlock;
use executor::ExecutorImpl;
use errors;
use function_optimizer::FunctionOptimizer;
use value::Value;

pub enum Function {
    Virtual(RefCell<VirtualFunction>),
    Native(NativeFunction)
}

pub struct VirtualFunction {
    basic_blocks: Vec<BasicBlock>,
    rt_handles: Vec<usize>,
    should_optimize: bool,
    this: Option<Value>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VirtualFunctionInfo {
    pub basic_blocks: Vec<BasicBlock>
}

pub type NativeFunction = Box<dyn Fn(&mut ExecutorImpl) -> Value + Send>;

impl Object for Function {
    fn initialize(&mut self, pool: &mut ObjectPool) {
        self.static_optimize(pool);
    }

    fn get_children(&self) -> Vec<usize> {
        match *self {
            Function::Virtual(ref f) => f.borrow().rt_handles.clone(),
            Function::Native(_) => Vec::new()
        }
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }

    fn call(&self, executor: &mut ExecutorImpl) -> Value {
        match *self {
            Function::Virtual(ref vf) => {
                let vf = vf.borrow();
                if let Some(this) = vf.this {
                    executor.get_current_frame().set_this(this);
                }
                executor.eval_basic_blocks(vf.basic_blocks.as_slice(), 0)
            },
            Function::Native(ref nf) => {
                nf(executor)
            }
        }
    }
}

impl Function {
    pub fn from_basic_blocks(blocks: Vec<BasicBlock>) -> Function {
        let vf = VirtualFunction {
            basic_blocks: blocks,
            rt_handles: Vec::new(),
            should_optimize: false,
            this: None
        };

        vf.validate().unwrap_or_else(|e| {
            panic!(errors::VMError::from(e))
        });

        Function::Virtual(RefCell::new(vf))
    }

    pub fn bind_this(&self, this: Value) {
        if let Function::Virtual(ref f) = *self {
            if let Ok(mut f) = f.try_borrow_mut() {
                if f.this.is_some() {
                    panic!(errors::VMError::from("Cannot rebind this"));
                }
                f.this = Some(this);
                if let Value::Object(id) = this {
                    f.rt_handles.push(id);
                }
            } else {
                panic!(errors::VMError::from("Cannot bind from inside the function"));
            }
        } else {
            panic!(errors::VMError::from("Binding this is only supported on virtual functions"));
        }
    }

    pub fn enable_optimization(&mut self) {
        if let Function::Virtual(ref mut f) = *self {
            f.borrow_mut().should_optimize = true;
        }
    }

    pub fn from_native(nf: NativeFunction) -> Function {
        Function::Native(nf)
    }

    pub fn to_virtual_info(&self) -> Option<VirtualFunctionInfo> {
        match *self {
            Function::Virtual(ref vf) => Some(VirtualFunctionInfo {
                basic_blocks: vf.borrow().basic_blocks.clone()
            }),
            Function::Native(_) => None
        }
    }

    pub fn from_virtual_info(vinfo: VirtualFunctionInfo) -> Self {
        Function::from_basic_blocks(vinfo.basic_blocks)
    }

    pub fn static_optimize(&self, pool: &mut ObjectPool) {
        if let Function::Virtual(ref f) = *self {
            if let Ok(mut f) = f.try_borrow_mut() {
                if f.should_optimize {
                    f.static_optimize(pool);
                }
            } else {
                panic!(errors::VMError::from("Cannot optimize virtual functions within itself"));
            }
        }
    }

    pub fn dynamic_optimize(&self, pool: &mut ObjectPool) {
        if let Function::Virtual(ref f) = *self {
            if let Ok(mut f) = f.try_borrow_mut() {
                if f.should_optimize {
                    f.dynamic_optimize(pool);
                }
            } else {
                panic!(errors::VMError::from("Cannot optimize virtual functions within itself"));
            }
        }
    }
}

impl VirtualFunction {
    fn static_optimize(&mut self, pool: &mut ObjectPool) {
        let mut optimizer = FunctionOptimizer::new(&mut self.basic_blocks, &mut self.rt_handles, pool);
        optimizer.set_binded_this(self.this);
        optimizer.static_optimize();
    }

    fn dynamic_optimize(&mut self, pool: &mut ObjectPool) {
        let mut optimizer = FunctionOptimizer::new(&mut self.basic_blocks, &mut self.rt_handles, pool);
        optimizer.set_binded_this(self.this);
        optimizer.dynamic_optimize();
    }

    pub fn validate(&self) -> Result<(), errors::ValidateError> {
        self.validate_basic_blocks()?;
        self.validate_branch_targets()?;
        Ok(())
    }

    pub fn validate_basic_blocks(&self) -> Result<(), errors::ValidateError> {
        for bb in &self.basic_blocks {
            bb.validate(false)?;
        }

        Ok(())
    }

    pub fn validate_branch_targets(&self) -> Result<(), errors::ValidateError> {
        let blocks = &self.basic_blocks;

        for bb in blocks {
            let mut found_error: bool = false;

            let (fst, snd) = bb.branch_targets();
            if let Some(fst) = fst {
                if fst >= blocks.len() {
                    found_error = true;
                }
            }
            if let Some(snd) = snd {
                if snd >= blocks.len() {
                    found_error = true;
                }
            }

            if found_error {
                return Err(errors::ValidateError::new("Invalid branch target(s)"));
            }
        }

        Ok(())
    }
}
