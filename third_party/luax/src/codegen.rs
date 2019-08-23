use std::prelude::v1::*;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use hexagon::basic_block::BasicBlock;
use hexagon::opcode::OpCode;
use hexagon::function::Function;
use ast;
use ast::GetEscapeInfo;
use ast_codegen::{UnrestrictedGenerateCode, CodegenError};

pub struct ModuleBuilder {
    scopes: RefCell<Vec<Scope>>,
    pub(crate) functions: RefCell<Vec<Function>>,
    next_unique_id: Cell<usize>
}

pub struct Scope {
    vars: HashMap<String, VarLocation>,
    is_function_root: bool
}

#[derive(Clone, Debug)]
pub enum VarLocation {
    Local(usize),
    This(String)
}

#[derive(Clone)]
pub struct LoopControlInfo {
    pub break_point: usize,
    pub continue_point: usize
}

pub struct FunctionBuilder<'a> {
    module: &'a ModuleBuilder,
    pub(crate) basic_blocks: Vec<BasicBlockBuilder>,
    next_local_id: usize,
    pub(crate) current_basic_block: usize,
    loop_control_info: Vec<LoopControlInfo>,
    closure_escaped_vars: HashSet<String>
}

pub struct BasicBlockBuilder {
    pub opcodes: Vec<OpCode>
}

impl BasicBlockBuilder {
    pub fn new() -> BasicBlockBuilder {
        BasicBlockBuilder {
            opcodes: Vec::new()
        }
    }

    pub fn detach_opcodes(&mut self, start: usize) -> Vec<OpCode> {
        self.opcodes.split_off(start)
    }
}

impl ModuleBuilder {
    pub fn new() -> ModuleBuilder {
        ModuleBuilder {
            scopes: RefCell::new(Vec::new()),
            functions: RefCell::new(Vec::new()),
            next_unique_id: Cell::new(0)
        }
    }

    pub fn new_function<'a>(&'a self) -> FunctionBuilder<'a> {
        FunctionBuilder::new(self)
    }

    pub fn push_scope(&self, scope: Scope) {
        self.scopes.borrow_mut().push(scope);
    }

    pub fn pop_scope(&self) -> Option<Scope> {
        self.scopes.borrow_mut().pop()
    }

    fn add_var_to_scope(&self, k: String, v: VarLocation) {
        let mut scopes = self.scopes.borrow_mut();
        let target_id = scopes.len() - 1;
        scopes[target_id].vars.insert(k, v);
    }

    pub fn lookup_var(&self, key: &str) -> Option<VarLocation> {
        let scopes = self.scopes.borrow();
        let mut passed_function_root: bool = false;

        for scope in scopes.iter().rev() {
            if let Some(ref loc) = scope.vars.get(key) {
                if passed_function_root {
                    // Accessing locals across a function boundary
                    // is invalid.
                    if let VarLocation::Local(_) = **loc {
                        continue
                    }
                }
                return Some((*loc).clone());
            }

            if scope.is_function_root {
                passed_function_root = true;
            }
        }

        None
    }

    pub fn get_unique_id(&self) -> String {
        let id = self.next_unique_id.get();
        self.next_unique_id.set(id + 1);

        format!("@__luax_internal.unique.{}", id)
    }
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            vars: HashMap::new(),
            is_function_root: false
        }
    }

    pub fn mark_as_function_root(&mut self) {
        self.is_function_root = true;
    }
}

impl<'a> FunctionBuilder<'a> {
    pub fn new(module: &'a ModuleBuilder) -> FunctionBuilder<'a> {
        let mut scope = Scope::new();
        scope.mark_as_function_root();

        module.push_scope(scope);

        FunctionBuilder {
            module: module,
            basic_blocks: vec! [
                BasicBlockBuilder::new(),
                BasicBlockBuilder::new()
            ],
            next_local_id: 0,
            current_basic_block: 1,
            loop_control_info: Vec::new(),
            closure_escaped_vars: HashSet::new()
        }
    }

    pub fn get_module_builder(&self) -> &ModuleBuilder {
        self.module
    }

    pub fn get_current_bb(&mut self) -> &mut BasicBlockBuilder {
        &mut self.basic_blocks[self.current_basic_block]
    }

    pub fn move_forward(&mut self) {
        self.basic_blocks.push(BasicBlockBuilder::new());
        self.current_basic_block += 1;
    }

    fn build_args_load(&mut self, names: Vec<String>) -> Result<(), CodegenError> {
        for i in 0..names.len() {
            let loc = self.create_local(names[i].as_str());
            self.get_current_bb().opcodes.push(
                OpCode::GetArgument(i)
            );
            loc.build_set(self)?;
        }
        Ok(())
    }

    pub fn create_local(&mut self, key: &str) -> VarLocation {
        let loc = if self.closure_escaped_vars.contains(key) {
            VarLocation::This(self.module.get_unique_id())
        } else {
            let v = VarLocation::Local(self.next_local_id);
            self.next_local_id += 1;
            v
        };
        self.module.add_var_to_scope(
            key.to_string(),
            loc.clone()
        );
        println!("[create_local] {} -> {:?}", key, loc);
        loc
    }

    pub fn get_var_location(&mut self, key: &str) -> VarLocation {
        let loc = match self.module.lookup_var(key) {
            Some(v) => v,
            None => VarLocation::This(key.to_string())
        };
        println!("[get_var_location] {} -> {:?}", key, loc);
        loc
    }

    pub fn get_anonymous_local(&mut self) -> VarLocation {
        let loc = VarLocation::Local(self.next_local_id);
        self.next_local_id += 1;

        loc
    }

    pub fn write_function_load(&mut self, id: usize) -> Result<(), CodegenError> {
        self.get_current_bb().opcodes.extend(vec! [
            OpCode::LoadInt(id as i64),
            OpCode::LoadString("__get__".into()),
            OpCode::LoadNull,
            OpCode::LoadString("@__luax_internal.functions".into()),
            OpCode::LoadThis,
            OpCode::GetField,
            OpCode::CallField(1)
        ]);
        Ok(())
    }

    pub fn write_array_create(&mut self) -> Result<(), CodegenError> {
        self.get_current_bb().opcodes.extend(vec! [
            OpCode::LoadNull,
            OpCode::LoadString("@__luax_internal.new_array".into()),
            OpCode::LoadThis,
            OpCode::GetField,
            OpCode::Call(0)
        ]);
        Ok(())
    }

    pub fn write_array_push(&mut self) -> Result<(), CodegenError> {
        self.get_current_bb().opcodes.extend(vec! [
            OpCode::LoadString("push".into()),
            OpCode::LoadNull,
            OpCode::Rotate3,
            OpCode::CallField(1),
            OpCode::Pop
        ]);
        Ok(())
    }

    pub fn write_table_create(&mut self) -> Result<(), CodegenError> {
        self.get_current_bb().opcodes.extend(vec! [
            OpCode::LoadNull,
            OpCode::LoadString("@__luax_internal.new_table".into()),
            OpCode::LoadThis,
            OpCode::GetField,
            OpCode::Call(0)
        ]);
        Ok(())
    }

    pub fn write_table_set(&mut self) -> Result<(), CodegenError> {
        self.get_current_bb().opcodes.extend(vec! [
            OpCode::LoadString("__set__".into()),
            OpCode::LoadNull,
            OpCode::Rotate3,
            OpCode::CallField(2),
            OpCode::Pop
        ]);
        Ok(())
    }

    pub fn write_concat(&mut self) -> Result<(), CodegenError> {
        self.get_current_bb().opcodes.extend(vec! [
            OpCode::StringAdd
        ]);
        Ok(())
    }

    pub fn write_pair_create(&mut self) -> Result<(), CodegenError> {
        self.get_current_bb().opcodes.extend(vec! [
            OpCode::LoadNull,
            OpCode::LoadString("@__luax_internal.new_pair".into()),
            OpCode::LoadThis,
            OpCode::GetField,
            OpCode::Call(2)
        ]);
        Ok(())
    }

    pub fn write_index_get(&mut self) -> Result<(), CodegenError> {
        self.get_current_bb().opcodes.extend(vec! [
            OpCode::LoadString("__get__".into()),
            OpCode::LoadNull,
            OpCode::Rotate3,
            OpCode::CallField(1)
        ]);
        Ok(())
    }

    pub fn write_index_set(&mut self) -> Result<(), CodegenError> {
        self.get_current_bb().opcodes.extend(vec! [
            OpCode::LoadString("__set__".into()),
            OpCode::LoadNull,
            OpCode::Rotate3,
            OpCode::CallField(2),
            OpCode::Pop
        ]);
        Ok(())
    }

    pub fn write_break(&mut self) -> Result<(), CodegenError> {
        let target = match self.get_lci() {
            Some(v) => v.break_point,
            None => return Err("No LCI".into())
        };
        self.get_current_bb().opcodes.push(OpCode::Branch(target));
        self.move_forward();
        Ok(())
    }

    pub fn write_continue(&mut self) -> Result<(), CodegenError> {
        let target = match self.get_lci() {
            Some(v) => v.continue_point,
            None => return Err("No LCI".into())
        };
        self.get_current_bb().opcodes.push(OpCode::Branch(target));
        self.move_forward();
        Ok(())
    }

    pub fn get_lci(&self) -> Option<&LoopControlInfo> {
        if !self.loop_control_info.is_empty() {
            Some(&self.loop_control_info[self.loop_control_info.len() - 1])
        } else {
            None
        }
    }

    pub fn with_lci<R, T: FnMut(&mut Self) -> R>(&mut self, lci: LoopControlInfo, mut f: T) -> R {
        self.loop_control_info.push(lci);
        let ret = catch_unwind(AssertUnwindSafe(|| f(self)));
        self.loop_control_info.pop().unwrap();

        match ret {
            Ok(v) => v,
            Err(e) => resume_unwind(e)
        }
    }

    pub fn scoped<R, T: FnMut(&mut Self) -> R>(&mut self, mut f: T) -> R {
        self.module.push_scope(Scope::new());
        let ret = catch_unwind(AssertUnwindSafe(|| f(self)));
        self.module.pop_scope();

        match ret {
            Ok(v) => v,
            Err(e) => resume_unwind(e)
        }
    }

    pub fn build(mut self, blk: &ast::Block, arg_names: Vec<String>) -> Result<usize, CodegenError> {
        self.closure_escaped_vars = blk.get_closure_escaped_vars().into_iter().collect();
        println!("Locals escaped to closures: {:?}", self.closure_escaped_vars);

        println!("{:?}", blk);

        self.build_args_load(arg_names)?;

        blk.unrestricted_generate_code(&mut self)?;
        self.get_current_bb().opcodes.push(OpCode::LoadNull);
        self.get_current_bb().opcodes.push(OpCode::Return);

        let n_locals = self.next_local_id;
        self.basic_blocks[0].opcodes = vec! [
            OpCode::InitLocal(n_locals),
            OpCode::Branch(1)
        ];
        let target_bbs: Vec<BasicBlock> = self.basic_blocks.iter()
            .map(|bb| BasicBlock::from_opcodes(bb.opcodes.clone()))
            .collect();
        let f = Function::from_basic_blocks(target_bbs);
        println!("{:?}", f.to_virtual_info().unwrap());

        let mut functions = self.module.functions.borrow_mut();
        functions.push(f);
        Ok(functions.len() - 1)
    }
}

impl<'a> Drop for FunctionBuilder<'a> {
    fn drop(&mut self) {
        self.module.pop_scope();
    }
}

impl VarLocation {
    pub fn build_get(&self, fb: &mut FunctionBuilder) -> Result<(), CodegenError> {
        match *self {
            VarLocation::Local(id) => {
                fb.get_current_bb().opcodes.push(
                    OpCode::GetLocal(id)
                );
                Ok(())
            },
            VarLocation::This(ref key) => {
                fb.get_current_bb().opcodes.extend(vec! [
                    OpCode::LoadString(key.clone()),
                    OpCode::LoadThis,
                    OpCode::GetField
                ]);
                Ok(())
            }
        }
    }

    pub fn build_set(&self, fb: &mut FunctionBuilder) -> Result<(), CodegenError> {
        match *self {
            VarLocation::Local(id) => {
                fb.get_current_bb().opcodes.push(
                    OpCode::SetLocal(id)
                );
                Ok(())
            },
            VarLocation::This(ref key) => {
                fb.get_current_bb().opcodes.extend(vec! [
                    OpCode::LoadString(key.clone()),
                    OpCode::LoadThis,
                    OpCode::SetField
                ]);
                Ok(())
            }
        }
    }
}
