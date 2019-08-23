#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::prelude::v1::*;
use ::Result;
use ast::*;
use instruction::*;
use std::collections::HashMap;
use std::f64;
use std::fmt::{Debug, Error as FmtError, Formatter};
use std::mem::swap;
use std::rc::Rc;
use std::result::Result as StdResult;
use value::*;

const MAX_REGISTERS: i32 = 200;
const REG_UNDEFINED: usize = OPCODE_MAXA as usize;
const LABEL_NO_JUMP: usize = 0;
const FIELDS_PER_FLUSH: i32 = 50;

fn lua_modulo(lhs: f64, rhs: f64) -> f64 {
    let mut v = lhs % rhs;
    if lhs < 0.0 || rhs < 0.0 && !(lhs < 0.0 && rhs < 0.0) {
        v += rhs;
    }
    v
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum ExprScope {
    Global,
    Upval,
    Local,
    Table,
    Vararg,
    Method,
    None,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct ExprContext {
    scope: ExprScope,
    reg: usize,
    /// opt >= 0: wants varargopt+1 results, i.e  a = func()
    /// opt = -1: ignore results             i.e  func()
    /// opt = -2: receive all results        i.e  a = {func()}
    opt: i32,
}

impl ExprContext {
    fn new(scope: ExprScope, reg: usize, opt: i32) -> ExprContext {
        ExprContext {
            scope,
            reg,
            opt,
        }
    }

    fn with_opt(opt: i32) -> ExprContext {
        ExprContext::new(ExprScope::None, REG_UNDEFINED, opt)
    }

    fn update(&mut self, typ: ExprScope, reg: usize, opt: i32) {
        self.scope = typ;
        self.reg = reg;
        self.opt = opt;
    }

    fn savereg(&self, reg: usize) -> usize {
        if self.scope != ExprScope::Local || self.reg == REG_UNDEFINED {
            reg
        } else {
            self.reg
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct AssignContext {
    expr_ctx: ExprContext,
    keyrk: usize,
    valrk: usize,
    keyks: bool,
    /// need move
    nmove: bool,
}

impl AssignContext {
    fn new(expr_ctx: ExprContext, keyrk: usize, valrk: usize, keyks: bool, nmove: bool) -> AssignContext {
        AssignContext {
            expr_ctx,
            keyrk,
            valrk,
            keyks,
            nmove,
        }
    }
}

#[derive(Debug)]
struct Lblabels {
    t: i32,
    f: i32,
    e: i32,
    b: bool,
}

impl Lblabels {
    fn new(t: i32, f: i32, e: i32, b: bool) -> Lblabels {
        Lblabels {
            t,
            f,
            e,
            b,
        }
    }
}

fn start_line<T>(p: &Node<T>) -> u32 {
    debug_assert!(p.line() != 0);
    p.line()
}

fn end_line<T>(p: &Node<T>) -> u32 {
    debug_assert!(p.last_line() != 0);
    p.last_line()
}

fn int2fb(val: i32) -> i32 {
    let mut e = 0;
    let mut x = val;
    while x >= 16 {
        x = (x + 1) >> 1;
        e += 1;
    }
    if x < 8 {
        return x;
    }
    ((e + 1) << 3) | (x - 8)
}

fn get_expr_name(expr: &Expr) -> String {
    match expr {
        Expr::Ident(ref s) => s.clone(),
        Expr::AttrGet(ref key, _) => {
            match key.inner() {
                Expr::String(ref s) => s.clone(),
                _ => "?".to_string()
            }
        }
        _ => "?".to_string()
    }
}

struct Instructions {
    insts: Vec<u32>,
    lines: Vec<u32>,
    pc: usize,
}

impl Instructions {
    fn new() -> Instructions {
        Instructions {
            insts: Vec::new(),
            lines: Vec::new(),
            pc: 0,
        }
    }

    fn add(&mut self, inst: Instruction, line: u32) {
        let len = self.insts.len();
        if len <= 0 || self.pc == len {
            self.insts.push(inst);
            self.lines.push(line);
        } else {
            let pc = self.pc;
            self.insts[pc] = inst;
            self.lines[pc] = line;
        }
        self.pc += 1;
    }

    fn add_ABC(&mut self, op: OpCode, a: i32, b: i32, c: i32, line: u32) {
        self.add(ABC(op, a, b, c), line);
    }

    fn add_ABx(&mut self, op: OpCode, a: i32, bx: i32, line: u32) {
        self.add(ABx(op, a, bx), line)
    }

    fn add_ASBx(&mut self, op: OpCode, a: i32, sbx: i32, line: u32) {
        self.add(ASBx(op, a, sbx), line)
    }

    fn set_opcode(&mut self, pc: usize, op: OpCode) {
        set_opcode(&mut self.insts[pc], op)
    }

    fn set_arga(&mut self, pc: usize, a: i32) {
        set_arga(&mut self.insts[pc], a)
    }

    fn set_argb(&mut self, pc: usize, b: i32) {
        set_argb(&mut self.insts[pc], b)
    }

    fn set_argc(&mut self, pc: usize, c: i32) {
        set_argc(&mut self.insts[pc], c)
    }

    fn set_argbx(&mut self, pc: usize, bx: i32) {
        set_argbx(&mut self.insts[pc], bx)
    }

    fn set_argsbx(&mut self, pc: usize, sbx: i32) {
        set_argsbx(&mut self.insts[pc], sbx)
    }

    fn at(&self, pc: usize) -> Instruction {
        self.insts[pc]
    }

    fn list(&self) -> Vec<Instruction> {
        let pc = self.pc;
        Vec::from(&self.insts[..pc])
    }

    fn line_list(&self) -> Vec<u32> {
        let pc = self.pc;
        Vec::from(&self.lines[..pc])
    }

    fn pc(&self) -> usize {
        self.pc
    }

    fn last_pc(&self) -> usize {
        self.pc - 1
    }

    fn last(&self) -> Instruction {
        if self.pc == 0 {
            INVALID_INSTRUCTION
        } else {
            self.insts[self.pc - 1]
        }
    }

    fn pop(&mut self) {
        self.pc -= 1
    }

    fn propagate_KMV(&mut self, top: usize, save: &mut usize, reg: &mut usize, inc: usize, loadk: bool) {
        let lastinst = self.last();
        if get_arga(lastinst) >= (top as i32) {
            match get_opcode(lastinst) {
                OP_LOADK => {
                    // if check `LOADK`
                    if loadk {
                        let cindex = get_argbx(lastinst);
                        if cindex <= opMaxIndexRk {
                            self.pop();
                            *save = rk_ask(cindex) as usize;
                            return;
                        }
                    }
                }
                OP_MOVE => {
                    self.pop();
                    *save = get_argb(lastinst) as usize;
                    return;
                }
                _ => {}
            }
        }
        *save = *reg;
        *reg += inc;
    }
}

impl Debug for Instructions {
    fn fmt(&self, f: &mut Formatter) -> StdResult<(), FmtError> {
        writeln!(f, "PC: <{}>", self.pc)?;
        for (i, inst) in self.insts.iter().enumerate() {
            writeln!(f, "<{:04}:L{:04}> {}", i, self.lines[i], to_string(*inst))?;
        }
        Ok(())
    }
}

/// Variable
#[derive(Debug)]
struct Var {
    index: usize,
    name: String,
}

impl Var {
    fn new(index: usize, name: String) -> Var {
        Var {
            index,
            name,
        }
    }
}

/// Variable table
#[derive(Debug, PartialEq)]
struct VarRegistry {
    names: Vec<String>,
    offset: usize,
}

impl VarRegistry {
    fn new(offset: usize) -> VarRegistry {
        VarRegistry {
            names: Vec::new(),
            offset,
        }
    }

    fn names(&self) -> Vec<String> {
        self.names.clone()
    }

    fn list(&self) -> Vec<Var> {
        self.names.
            iter().
            enumerate().
            map(|(index, name)|
                Var::new(index + self.offset, name.clone())).
            collect()
    }

    fn last_index(&self) -> usize {
        self.offset + self.names.len()
    }

    fn find(&self, name: &String) -> Option<usize> {
        self.names
            .iter()
            .enumerate()
            .rev()
            .find(|x| x.1 == name)
            .map(|x| self.offset + x.0)
    }

    fn register_unique(&mut self, name: String) -> usize {
        match self.find(&name) {
            Some(index) => index,
            None => self.register(name),
        }
    }

    fn register(&mut self, name: String) -> usize {
        self.names.push(name);
        self.names.len() - 1 + self.offset
    }
}

#[derive(Debug, PartialEq)]
struct Block {
    locals: VarRegistry,
    break_label: usize,
    parent: Option<Box<Block>>,
    ref_upval: bool,
    lineinfo: (u32, u32),
}

impl Block {
    fn new(locals: VarRegistry, break_label: usize, lineinfo: (u32, u32)) -> Box<Block> {
        Box::new(Block {
            locals,
            break_label,
            parent: None,
            ref_upval: false,
            lineinfo,
        })
    }

    /// Recursively find a local variable, return a block
    /// containing a local variable
    fn find_var_block(&mut self, name: &String) -> Option<&mut Block> {
        match self.locals.find(name) {
            Some(_) => Some(self),
            None => match self.parent {
                Some(ref mut parent) => parent.find_var_block(name),
                None => None
            }
        }
    }

    fn set_parent(&mut self, parent: Option<Box<Block>>) {
        self.parent = parent;
    }

    fn set_ref_upval(&mut self, b: bool) {
        self.ref_upval = b;
    }

    fn startline(&self) -> u32 {
        self.lineinfo.0
    }

    fn endline(&self) -> u32 {
        self.lineinfo.1
    }
}

const VARARG_HAS: u8 = 1;
const VARARG_IS: u8 = 2;
const VARARG_NEED: u8 = 4;

#[derive(Debug)]
struct DebugLocalInfo {
    name: String,
    /// start pc
    spc: usize,
    /// end pc
    epc: usize,
}

impl DebugLocalInfo {
    fn new(name: String, spc: usize, epc: usize) -> Box<DebugLocalInfo> {
        Box::new(DebugLocalInfo {
            name,
            spc,
            epc,
        })
    }
}

struct DebugCall {
    name: String,
    pc: usize,
}

impl DebugCall {
    fn new(name: String, pc: usize) -> DebugCall {
        DebugCall {
            name,
            pc,
        }
    }
}

impl Debug for DebugCall {
    fn fmt(&self, f: &mut Formatter) -> StdResult<(), FmtError> {
        write!(f, "<{:04}> {}", self.pc, self.name)
    }
}

#[derive(Debug)]
pub struct FunctionProto {
    source: String,
    /// lineinfo: (startline, endline)
    lineinfo: (u32, u32),
    upval_count: u8,
    param_count: u8,
    is_vararg: u8,
    used_registers: u8,
    code: Vec<Instruction>,
    constants: Vec<Rc<Value>>,
    prototypes: Vec<Box<FunctionProto>>,

    /// Debug information
    debug_pos: Vec<u32>,
    debug_locals: Vec<Box<DebugLocalInfo>>,
    debug_calls: Vec<DebugCall>,
    debug_upval: Vec<String>,

    /// String constants
    strings: Vec<String>,
}

impl FunctionProto {
    pub fn new(source: String) -> Box<FunctionProto> {
        Box::new(FunctionProto {
            source,
            lineinfo: (0, 0),
            upval_count: 0,
            param_count: 0,
            is_vararg: 0,
            used_registers: 2,
            code: Vec::with_capacity(128),
            constants: Vec::with_capacity(32),
            prototypes: Vec::with_capacity(16),
            debug_pos: Vec::with_capacity(128),
            debug_locals: Vec::with_capacity(16),
            debug_calls: Vec::with_capacity(128),
            debug_upval: Vec::with_capacity(16),
            strings: Vec::with_capacity(32),
        })
    }
}

#[derive(Debug)]
struct Compiler<'p> {
    proto: Box<FunctionProto>,
    code: Instructions,
    parent: Option<&'p Compiler<'p>>,
    upval: VarRegistry,
    block: Box<Block>,

    reg_top: usize,
    label_id: i32,
    label_pc: HashMap<i32, usize>,
}

impl<'p> Compiler<'p> {
    fn new(source: String, parent: Option<&'p Compiler>) -> Box<Compiler<'p>> {
        Box::new(Compiler {
            proto: FunctionProto::new(source),
            code: Instructions::new(),
            parent,
            upval: VarRegistry::new(0),
            block: Block::new(VarRegistry::new(0), LABEL_NO_JUMP, (0, 0)),
            reg_top: 0,
            label_id: 1,
            label_pc: HashMap::new(),
        })
    }

    fn new_label(&mut self) -> i32 {
        let r = self.label_id;
        self.label_id += 1;
        r
    }

    fn set_label_pc(&mut self, label: i32, pc: usize) {
        self.label_pc.insert(label, pc);
    }

    fn get_label_pc(&self, label: i32) -> usize {
        self.label_pc[&label]
    }

    fn const_index(&mut self, value: Rc<Value>) -> usize {
        let v = self.proto.constants
            .iter()
            .enumerate()
            .find(|x| x.1 == &value)
            .map(|x| x.0);

        match v {
            Some(v) => v,
            None => {
                self.proto.constants.push(value);
                let index = self.proto.constants.len() - 1;
                if index > (OPCODE_MAXBx as usize) {
                    panic!("{}:{:?} to many constants", self.proto.source, self.proto.lineinfo)
                } else {
                    index
                }
            }
        }
    }

    fn reg_top(&self) -> usize {
        self.reg_top
    }

    fn register_local_var(&mut self, name: String) -> usize {
        let ret = self.block.locals.register(name.clone());
        self.proto.debug_locals.push(DebugLocalInfo::new(name, self.code.pc(), 0));
        self.reg_top += 1;
        ret
    }

    fn find_local_var(&self, name: &String) -> Option<usize> {
        let mut blk = &self.block;
        loop {
            let r = blk.locals.find(name);
            match r {
                Some(i) => return Some(i),
                None => match blk.parent {
                    Some(ref parent) => blk = parent,
                    None => return None
                }
            }
        }
    }

    fn enter_block(&mut self, blabel: usize, lineinfo: (u32, u32)) {
        let vtb = VarRegistry::new(self.reg_top());
        let mut blk = Block::new(vtb, blabel, lineinfo);
        swap(&mut blk, &mut self.block);
        self.block.set_parent(Some(blk));
    }

    fn close_upval(&mut self) -> Option<usize> {
        if self.block.ref_upval {
            match self.block.parent {
                Some(ref parent) => {
                    let x = parent.locals.last_index();
                    self.code.add_ABC(OP_CLOSE, x as i32, 0, 0, self.block.endline());
                    Some(x)
                }
                None => None,
            }
        } else {
            None
        }
    }

    fn end_scope(&mut self) {
        let last_pc = self.code.last_pc();
        for vr in self.block.locals.list().iter() {
            self.proto.debug_locals[vr.index].epc = last_pc;
        }
    }

    fn leave_block(&mut self) -> Option<usize> {
        let closed = self.close_upval();
        self.end_scope();
        let mut parent: Option<Box<Block>> = None; // swap replacement
        swap(&mut self.block.parent, &mut parent);
        self.block = parent.unwrap();
        self.reg_top = self.block.locals.last_index();
        closed
    }

    fn compile_chunk(&mut self, chunk: &Vec<StmtNode>) {
        for stmt in chunk.iter() {
            self.compile_stmt(stmt);
        }
    }

    fn compile_block(&mut self, block: &Vec<StmtNode>) {
        if block.len() < 1 {
            return;
        }
        let start_line = start_line(&block[0]);
        let end_line = end_line(&block[block.len() - 1]);
        self.enter_block(LABEL_NO_JUMP, (start_line, end_line));
        for stmt in block.iter() {
            self.compile_stmt(stmt);
        }
        self.leave_block();
    }

    fn get_ident_reftype(&self, name: &String) -> ExprScope {
        // local variable
        match self.find_local_var(name) {
            Some(_) => ExprScope::Local,
            None => {
                // upvalue or global variable
                let t = match self.parent {
                    Some(ref parent) => parent.get_ident_reftype(name),
                    None => ExprScope::Global,
                };

                if t != ExprScope::Global {
                    ExprScope::Upval
                } else {
                    ExprScope::Global
                }
            }
        }
    }

    fn load_rk(&mut self, reg: &mut usize, expr: &ExprNode, cnst: Rc<Value>) -> i32 {
        let cindex = self.const_index(cnst) as i32;
        if cindex < opMaxIndexRk {
            rk_ask(cindex)
        } else {
            let ret = *reg;
            *reg += 1;
            self.code.add_ABx(OP_LOADK, ret as i32, cindex, start_line(expr));
            ret as i32
        }
    }

    fn compile_table_expr(&mut self, mut reg: usize, table: &ExprNode, expr_ctx: &ExprContext) {
        let tablereg = reg;
        reg += 1;
        self.code.add_ABC(OP_NEWTABLE, tablereg as i32, 0, 0, start_line(table));
        let tablepc = self.code.last_pc();
        let regbase = reg;
        if let Expr::Table(ref fields) = table.inner() {
            let mut array_count = 0;
            let mut lastvarargs = false;
            let fieldlen = fields.len();
            for (i, field) in fields.iter().enumerate() {
                let islast = i == fieldlen - 1;
                match field.key {
                    None => {
                        if islast && field.val.inner().is_vararg() {
                            lastvarargs = true;
                            reg += self.compile_expr(reg, &field.val, &ExprContext::with_opt(-2));
                        } else {
                            array_count += 1;
                            reg += self.compile_expr(reg, &field.val, &ExprContext::with_opt(0))
                        }
                    }
                    Some(ref key) => {
                        let regorg = reg;
                        let mut b = reg;
                        self.compile_expr_with_KMV_propagation(&key, &mut reg, &mut b);
                        let mut c = reg;
                        self.compile_expr_with_KMV_propagation(&field.val, &mut reg, &mut c);
                        let opcode = if let Expr::String(_) = key.inner() { OP_SETTABLEKS } else { OP_SETTABLE };
                        self.code.add_ABC(opcode, tablereg as i32, b as i32, c as i32, start_line(key));
                        reg = regorg;
                    }
                }
                let flush = array_count % FIELDS_PER_FLUSH;
                if (array_count != 0 && (flush == 0 || islast)) || lastvarargs {
                    reg = regbase;
                    let num = if flush == 0 { FIELDS_PER_FLUSH } else { flush };
                    let mut c = (array_count - 1) / FIELDS_PER_FLUSH + 1;
                    let b = if islast && field.val.inner().is_vararg() { 0 } else { num };
                    let line = match field.key {
                        Some(ref expr) => start_line(expr),
                        None => start_line(&field.val),
                    };
                    if c > 511 {
                        c = 0;
                    }
                    self.code.add_ABC(OP_SETLIST, tablereg as i32, b as i32, c as i32, line);
                    if c == 0 {
                        self.code.add(0, line);
                    }
                }
            }

            self.code.set_argb(tablepc, int2fb(array_count as i32));
            self.code.set_argc(tablepc, int2fb(fieldlen as i32 - array_count));
            if expr_ctx.scope == ExprScope::Local && expr_ctx.reg != tablereg {
                self.code.add_ABC(OP_MOVE, expr_ctx.reg as i32, tablereg as i32, 0, start_line(table))
            }
        } else {
            unreachable!()
        }
    }

    fn compile_fncall_expr(&mut self, mut reg: usize, expr: &ExprNode, expr_ctx: &ExprContext) -> usize {
        if expr_ctx.scope == ExprScope::Local
            && self.proto.param_count > 0
            && expr_ctx.reg == self.proto.param_count as usize - 1 {
            reg = expr_ctx.reg
        }
        let funcreg = reg;
        let mut islastvargs = false;

        let (name, argc) = match expr.inner() {
            Expr::FuncCall(ref func) => {
                reg += self.compile_expr(reg, &func.func, &ExprContext::with_opt(0));
                let len = func.args.len();
                for (i, arg) in func.args.iter().enumerate() {
                    islastvargs = (i == len - 1) && arg.inner().is_vararg();
                    if islastvargs {
                        self.compile_expr(reg, arg, &ExprContext::with_opt(-2));
                    } else {
                        reg += self.compile_expr(reg, arg, &ExprContext::with_opt(0));
                    }
                }
                (get_expr_name(&func.func.inner()), len)
            }
            Expr::MethodCall(ref method) => {
                let mut b = reg;
                self.compile_expr_with_MV_propagation(&method.receiver, &mut reg, &mut b);
                let c = self.load_rk(&mut reg, expr, Rc::new(Value::String(method.method.clone())));
                self.code.add_ABC(OP_SELF, funcreg as i32, b as i32, c, start_line(expr));
                reg = b + 1;
                let reg2 = funcreg + 2;
                if reg2 > reg {
                    reg = reg2
                }
                let len = method.args.len();
                for (i, arg) in method.args.iter().enumerate() {
                    islastvargs = (i == len - 1) && arg.inner().is_vararg();
                    if islastvargs {
                        self.compile_expr(reg, arg, &ExprContext::with_opt(-2));
                    } else {
                        reg += self.compile_expr(reg, arg, &ExprContext::with_opt(0));
                    }
                }
                (method.method.clone(), len + 1)
            }
            _ => unreachable!()
        };

        let b = if islastvargs { 0 } else { argc + 1 };
        self.code.add_ABC(OP_CALL, funcreg as i32, b as i32, expr_ctx.opt + 2, start_line(expr));
        self.proto.debug_calls.push(DebugCall::new(name, self.code.last_pc()));

        if expr_ctx.opt == 0 && expr_ctx.scope == ExprScope::Local && funcreg != expr_ctx.reg {
            self.code.add_ABC(OP_MOVE, expr_ctx.reg as i32, funcreg as i32, 0, start_line(expr));
            return 1;
        }

        if self.reg_top() > (funcreg + (expr_ctx.opt + 2) as usize) || expr_ctx.opt < -1 {
            return 0;
        }

        (expr_ctx.opt + 1) as usize
    }

    fn compile_binary_arith_expr(&mut self, mut reg: usize,
                                 opr: BinaryOpr, lhs: &ExprNode, rhs: &ExprNode,
                                 expr_ctx: &ExprContext, line: u32) {
        let a = expr_ctx.savereg(reg);
        let mut b = reg;
        self.compile_expr_with_KMV_propagation(lhs, &mut reg, &mut b);
        let mut c = reg;
        self.compile_expr_with_KMV_propagation(rhs, &mut reg, &mut c);

        let opcode = match opr {
            BinaryOpr::Add => OP_ADD,
            BinaryOpr::Sub => OP_SUB,
            BinaryOpr::Mul => OP_MUL,
            BinaryOpr::Div => OP_DIV,
            BinaryOpr::Mod => OP_MOD,
            BinaryOpr::Pow => OP_POW,
            _ => unreachable!()
        };
        self.code.add_ABC(opcode, a as i32, b as i32, c as i32, line);
    }

    fn compile_binary_rel_expr_aux(&mut self, mut reg: usize,
                                   opr: BinaryOpr, lhs: &ExprNode, rhs: &ExprNode,
                                   flip: i32, jumplabel: i32, line: u32) {
        let mut b = reg;
        self.compile_expr_with_KMV_propagation(lhs, &mut reg, &mut b);
        let mut c = reg;
        self.compile_expr_with_KMV_propagation(rhs, &mut reg, &mut c);

        let inst = match opr {
            BinaryOpr::Eq => ABC(OP_EQ, 0 ^ flip, b as i32, c as i32),
            BinaryOpr::NE => ABC(OP_EQ, 1 ^ flip, b as i32, c as i32),
            BinaryOpr::LT => ABC(OP_LT, 0 ^ flip, b as i32, c as i32),
            BinaryOpr::GT => ABC(OP_LT, 0 ^ flip, c as i32, b as i32),
            BinaryOpr::LE => ABC(OP_LE, 0 ^ flip, b as i32, c as i32),
            BinaryOpr::GE => ABC(OP_LE, 0 ^ flip, c as i32, b as i32),
            _ => unreachable!()
        };
        self.code.add(inst, line);
        self.code.add_ASBx(OP_JMP, 0, jumplabel, line);
    }

    fn compile_binary_rel_expr(&mut self, reg: usize,
                               opr: BinaryOpr, lhs: &ExprNode, rhs: &ExprNode,
                               expr_ctx: &ExprContext, line: u32) {
        let a = expr_ctx.savereg(reg);
        let jumplabel = self.new_label();
        self.compile_binary_rel_expr_aux(reg, opr, lhs, rhs, 1, jumplabel, line);
        self.code.add_ABC(OP_LOADBOOL, a as i32, 0, 1, line);
        let lastpc = self.code.last_pc();
        self.set_label_pc(jumplabel, lastpc);
        self.code.add_ABC(OP_LOADBOOL, a as i32, 1, 0, line);
    }

    fn compile_binary_log_expr_aux(&mut self, reg: usize,
                                   expr: &ExprNode, expr_ctx: &ExprContext,
                                   thenlabel: i32, elselabel: i32, hasnextcond: bool, lb: &mut Lblabels) {
        let mut flip = 0;
        let mut jumplabel = elselabel;
        if hasnextcond {
            flip = 1;
            jumplabel = thenlabel;
        }

        match expr.inner() {
            &Expr::False => {
                if elselabel == lb.e {
                    self.code.add_ASBx(OP_JMP, 0, lb.f, start_line(expr));
                    lb.b = true;
                } else {
                    self.code.add_ASBx(OP_JMP, 0, elselabel, start_line(expr));
                }
            }
            &Expr::True => {
                if thenlabel == lb.e {
                    self.code.add_ASBx(OP_JMP, 0, lb.t, start_line(expr));
                    lb.b = true;
                } else {
                    self.code.add_ASBx(OP_JMP, 0, thenlabel, start_line(expr));
                }
            }
            &Expr::Nil => {
                if elselabel == lb.e {
                    self.compile_expr(reg, expr, expr_ctx);
                    self.code.add_ASBx(OP_JMP, 0, lb.e, start_line(expr));
                } else {
                    self.code.add_ASBx(OP_JMP, 0, elselabel, start_line(expr));
                }
            }
            &Expr::String(_) | &Expr::Number(_) => {
                if thenlabel == lb.e {
                    self.compile_expr(reg, expr, expr_ctx);
                    self.code.add_ASBx(OP_JMP, 0, lb.e, start_line(expr));
                } else {
                    self.code.add_ASBx(OP_JMP, 0, thenlabel, start_line(expr));
                }
            }
            &Expr::BinaryOp(BinaryOpr::And, ref lhs, ref rhs) => {
                let nextcondlabel = self.new_label();
                self.compile_binary_log_expr_aux(reg, lhs, expr_ctx, nextcondlabel, elselabel, false, lb);
                let lastpc = self.code.last_pc();
                self.set_label_pc(nextcondlabel, lastpc);
                self.compile_binary_log_expr_aux(reg, rhs, expr_ctx, thenlabel, elselabel, hasnextcond, lb);
            }
            &Expr::BinaryOp(BinaryOpr::Or, ref lhs, ref rhs) => {
                let nextcondlabel = self.new_label();
                self.compile_binary_log_expr_aux(reg, lhs, expr_ctx, thenlabel, nextcondlabel, true, lb);
                let lastpc = self.code.last_pc();
                self.set_label_pc(nextcondlabel, lastpc);
                self.compile_binary_log_expr_aux(reg, rhs, expr_ctx, thenlabel, elselabel, hasnextcond, lb);
            }
            Expr::BinaryOp(ref opr, ref lhs, ref rhs)
            if opr == &BinaryOpr::Eq ||
                opr == &BinaryOpr::LT ||
                opr == &BinaryOpr::LE ||
                opr == &BinaryOpr::NE ||
                opr == &BinaryOpr::GT ||
                opr == &BinaryOpr::GE => {
                if thenlabel == elselabel {
                    flip ^= 1;
                    jumplabel = lb.t;
                    lb.b = true;
                } else if thenlabel == lb.e {
                    jumplabel = lb.t;
                    lb.b = true;
                } else if elselabel == lb.e {
                    jumplabel = lb.f;
                    lb.b = true;
                }
                self.compile_binary_rel_expr_aux(reg, *opr, lhs, rhs, flip, jumplabel, start_line(expr));
            }
            _ => {
                if !hasnextcond && thenlabel == elselabel {
                    //reg += self.compile_expr(reg, expr, expr_ctx);
                } else {
                    let a = reg;
                    let sreg = expr_ctx.savereg(a);
                    //reg += self.compile_expr(reg, expr, &ExprContext::with_opt(0));
                    if sreg == a {
                        self.code.add_ABC(OP_TEST, a as i32, 0, 0 ^ flip, start_line(expr));
                    } else {
                        self.code.add_ABC(OP_TESTSET, sreg as i32, a as i32, 0 ^ flip, start_line(expr));
                    }
                }
                self.code.add_ASBx(OP_JMP, 0, jumplabel, start_line(expr))
            }
        }
    }

    fn compile_binary_log_expr(&mut self, reg: usize,
                               opr: BinaryOpr, lhs: &ExprNode, rhs: &ExprNode,
                               expr_ctx: &ExprContext, line: u32) {
        let a = expr_ctx.savereg(reg);
        let endlabel = self.new_label();
        let mut lb = Lblabels::new(self.new_label(), self.new_label(), endlabel, false);
        let nextcondlabel = self.new_label();
        match opr {
            BinaryOpr::And => {
                self.compile_binary_log_expr_aux(reg, lhs, expr_ctx, nextcondlabel, endlabel, false, &mut lb);
                let lastpc = self.code.last_pc();
                self.set_label_pc(nextcondlabel, lastpc);
                self.compile_binary_log_expr_aux(reg, rhs, expr_ctx, endlabel, endlabel, false, &mut lb);
            }
            BinaryOpr::Or => {
                self.compile_binary_log_expr_aux(reg, lhs, expr_ctx, endlabel, nextcondlabel, true, &mut lb);
                let lastpc = self.code.last_pc();
                self.set_label_pc(nextcondlabel, lastpc);
                self.compile_binary_log_expr_aux(reg, rhs, expr_ctx, endlabel, endlabel, false, &mut lb);
            }
            _ => unreachable!()
        }

        if lb.b {
            let lastpc = self.code.last_pc();
            self.set_label_pc(lb.f, lastpc);
            self.code.add_ABC(OP_LOADBOOL, a as i32, 0, 1, line);
            let lastpc = self.code.last_pc();
            self.set_label_pc(lb.t, lastpc);
            self.code.add_ABC(OP_LOADBOOL, a as i32, 1, 0, line);
        }

        let lastinst = self.code.last();
        if get_opcode(lastinst) == OP_JMP && get_argsbx(lastinst) == endlabel {
            self.code.pop();
        }
        let lastpc = self.code.last_pc();
        self.set_label_pc(endlabel, lastpc);
    }

    fn compile_binaryop_expr(&mut self, reg: usize, expr: &ExprNode, expr_ctx: &ExprContext) {
        match expr.inner() {
            Expr::BinaryOp(ref opr, ref lhs, ref rhs)
            if opr == &BinaryOpr::Add ||
                opr == &BinaryOpr::Sub ||
                opr == &BinaryOpr::Mul ||
                opr == &BinaryOpr::Div ||
                opr == &BinaryOpr::Mod ||
                opr == &BinaryOpr::Pow => {
                if let Some(e) = self.const_fold(expr) {
                    let exprnode = ExprNode::new(e, expr.lineinfo());
                    self.compile_expr(reg, &exprnode, expr_ctx);
                } else {
                    self.compile_binary_arith_expr(reg, *opr, lhs, rhs, expr_ctx, start_line(expr));
                }
            }
            Expr::BinaryOp(BinaryOpr::Concat, ref lhs, ref rhs) => {
                let mut crange = 1;
                let mut current = rhs;
                loop {
                    match current.inner() {
                        Expr::BinaryOp(BinaryOpr::Concat, ref sublhs, ref subrhs) => {
                            crange += 1;
                            current = subrhs;
                        }
                        _ => break
                    }
                }
                let a = expr_ctx.savereg(reg);
                let basereg = reg;
                //reg += self.compile_expr(reg, lhs, &ExprContext::with_opt(0));
                //reg += self.compile_expr(reg, rhs, &ExprContext::with_opt(0));
                let mut pc = self.code.last_pc();
                while pc != 0 && get_opcode(self.code.at(pc)) == OP_CONCAT {
                    self.code.pop();
                    pc -= 1;
                }
                self.code.add_ABC(OP_CONCAT, a as i32, basereg as i32, basereg as i32 + crange, start_line(expr));
            }
            Expr::BinaryOp(ref opr, ref lhs, ref rhs)
            if opr == &BinaryOpr::Eq ||
                opr == &BinaryOpr::LT ||
                opr == &BinaryOpr::LE ||
                opr == &BinaryOpr::NE ||
                opr == &BinaryOpr::GT ||
                opr == &BinaryOpr::GE => {
                self.compile_binary_rel_expr(reg, *opr, lhs, rhs, expr_ctx, start_line(expr));
            }
            Expr::BinaryOp(opr, ref lhs, ref rhs) if opr == &BinaryOpr::And || opr == &BinaryOpr::Or => {
                self.compile_binary_log_expr(reg, *opr, lhs, rhs, expr_ctx, start_line(expr));
            }
            _ => unreachable!()
        }
    }

    fn compile_unaryop_expr(&mut self, mut reg: usize, expr: &ExprNode, expr_ctx: &ExprContext) {
        let (opcode, operand) = match expr.inner() {
            Expr::UnaryOp(UnaryOpr::Not, ref subexpr) => {
                match subexpr.inner() {
                    &Expr::True => {
                        self.code.add_ABC(OP_LOADBOOL, expr_ctx.savereg(reg) as i32, 0, 0, start_line(expr));
                        return;
                    }
                    &Expr::False | &Expr::Nil => {
                        self.code.add_ABC(OP_LOADBOOL, expr_ctx.savereg(reg) as i32, 1, 0, start_line(expr));
                        return;
                    }
                    _ => (OP_NOT, subexpr)
                }
            }
            Expr::UnaryOp(UnaryOpr::Length, ref subexpr) => (OP_LEN, subexpr),
            Expr::UnaryOp(UnaryOpr::Minus, ref subexpr) => {
                if let Some(e) = self.const_fold(expr) {
                    let exprnode = ExprNode::new(e, expr.lineinfo());
                    self.compile_expr(reg, &exprnode, expr_ctx);
                    return;
                }
                (OP_UNM, subexpr)
            },
            _ => unreachable!()
        };

        let a = expr_ctx.savereg(reg);
        let mut b = reg;
        self.compile_expr_with_MV_propagation(operand, &mut reg, &mut b);
        self.code.add_ABC(opcode, a as i32, b as i32, 0, start_line(expr));
    }

    /// Attempts to constant fold (i.e. evaluate at compile time as an optimization) the given
    /// operation on the two expressions.
    /// Folding is attempted only on operations on numbers that are known constants
    /// (e.g. `local x = 1+1` -> `local x = 2`).  Moreover, constant folding is skipped on
    /// division (or modulo) by zero (resulting in not-a-number, NaN), which Lua 5.1 folded
    /// but it caused problems so this folding was eliminated in Lua 5.2.
    /// Returns 1 (not 0) if folding was possible.
    /// --see http://lua-users.org/lists/lua-l/2007-02/msg00207.html.
    fn const_fold(&mut self, expr: &ExprNode) -> Option<Expr> {
        match expr.inner() {
            Expr::UnaryOp(UnaryOpr::Minus, ref subexpr) => {
                match self.const_fold(subexpr) {
                    None => if let Expr::Number(n) = subexpr.inner() { Some(Expr::Number(-n)) } else { None }
                    Some(sub) => {
                        if let Expr::Number(n) = sub {
                            Some(Expr::Number(-n))
                        } else {
                            unreachable!()
                        }
                    }
                }
            }
            Expr::BinaryOp(ref opr, ref lhs, ref rhs)
            if opr == &BinaryOpr::Add ||
                opr == &BinaryOpr::Sub ||
                opr == &BinaryOpr::Mul ||
                opr == &BinaryOpr::Div ||
                opr == &BinaryOpr::Mod ||
                opr == &BinaryOpr::Pow => {
                if let Expr::Number(l) = lhs.inner() {
                    if let Expr::Number(r) = rhs.inner() {
                        let v = match opr {
                            &BinaryOpr::Add => { l + r }
                            &BinaryOpr::Sub => { l - r }
                            &BinaryOpr::Mul => { l * r }
                            &BinaryOpr::Div => { l / r }
                            &BinaryOpr::Mod => { lua_modulo(*l, *r) }
                            &BinaryOpr::Pow => { l.powf(*r) }
                            _ => unreachable!()
                        };
                        return Some(Expr::Number(v));
                    }
                }

                // TODO: partial fold
                let subl = Box::new(ExprNode::new(self.const_fold(lhs)?, lhs.lineinfo()));
                let subr = Box::new(ExprNode::new(self.const_fold(rhs)?, lhs.lineinfo()));
                Some(Expr::BinaryOp(*opr, subl, subr))
            }
            Expr::Number(n) => Some(Expr::Number(*n)),
            _ => None
        }
    }

    fn compile_expr(&mut self, mut reg: usize, expr: &ExprNode, expr_ctx: &ExprContext) -> usize {
        let sreg = expr_ctx.savereg(reg);
        let mut sused: usize = if sreg < reg { 0 } else { 1 };
        let svreg = sreg as i32;

        // TODO: const value
        match expr.inner() {
            &Expr::True => self.code.add_ABC(OP_LOADBOOL, svreg, 1, 0, start_line(expr)),
            &Expr::False => self.code.add_ABC(OP_LOADBOOL, svreg, 0, 0, start_line(expr)),
            &Expr::Nil => self.code.add_ABC(OP_LOADNIL, svreg, svreg, 0, start_line(expr)),
            &Expr::Number(f) => {
                let num = self.const_index(Rc::new(Value::Number(f)));
                self.code.add_ABx(OP_LOADK, svreg, num as i32, start_line(expr))
            }
            &Expr::String(ref s) => {
                let index = self.const_index(Rc::new(Value::String(s.clone())));
                self.code.add_ABx(OP_LOADK, svreg, index as i32, start_line(expr))
            }
            &Expr::Dots => {
                if self.proto.is_vararg == 0 {
                    panic!("cannot use '...' outside a vararg function")
                }
                self.proto.is_vararg &= !VARARG_NEED;
                self.code.add_ABC(OP_VARARG, svreg, expr_ctx.opt + 2, 0, start_line(expr));
                sused = if self.reg_top() > (expr_ctx.opt + 2) as usize || expr_ctx.opt < -1 {
                    0
                } else {
                    (svreg + 1 + expr_ctx.opt) as usize - reg
                }
            }
            &Expr::Ident(ref s) => {
                let identtype = self.get_ident_reftype(s);
                match identtype {
                    ExprScope::Global => {
                        let index = self.const_index(Rc::new(Value::String(s.clone())));
                        self.code.add_ABx(OP_GETGLOBAL, svreg, index as i32, start_line(expr))
                    }
                    ExprScope::Upval => {
                        let index = self.upval.register_unique(s.clone());
                        self.code.add_ABC(OP_GETUPVAL, svreg, index as i32, 0, start_line(expr));
                    }
                    ExprScope::Local => {
                        let index = self.find_local_var(s).unwrap() as i32;
                        self.code.add_ABC(OP_MOVE, svreg, index, 0, start_line(expr));
                    }
                    _ => unreachable!()
                }
            }
            &Expr::AttrGet(ref obj, ref key) => {
                let a = svreg;
                let mut b = reg.clone();
                self.compile_expr_with_MV_propagation(obj, &mut reg, &mut b);
                let mut c = reg.clone();
                self.compile_expr_with_KMV_propagation(key, &mut reg, &mut c);
                let opcode = if let &Expr::String(_) = key.inner() { OP_GETTABLEKS } else { OP_GETTABLE };
                self.code.add_ABC(opcode, a, b as i32, c as i32, start_line(expr));
            }
            &Expr::Table(_) => {
                self.compile_table_expr(reg, expr, expr_ctx);
                // TODO: needs?
                sused = 1;
            },
            &Expr::FuncCall(_) | &Expr::MethodCall(_) => sused = self.compile_fncall_expr(reg, expr, expr_ctx),
            &Expr::BinaryOp(_, _, _) => self.compile_binaryop_expr(reg, expr, expr_ctx),
            &Expr::UnaryOp(_, _) => self.compile_unaryop_expr(reg, expr, expr_ctx),
            &Expr::Function(ref params, ref stmts) => {
                let (proto, upvals) = {
                    let mut subcompiler = Compiler::new(self.proto.source.clone(), Some(self));
                    subcompiler.compile_func_expr(params, stmts, expr_ctx, expr.lineinfo());
                    let mut upval = VarRegistry::new(0);
                    swap(&mut subcompiler.upval, &mut upval);
                    (subcompiler.proto, upval)
                };

                let protono = self.proto.prototypes.len();
                self.proto.prototypes.push(proto);
                self.code.add_ABx(OP_CLOSURE, svreg, protono as i32, start_line(expr));
                for upv in &upvals.names {
                    let (op, mut index) = match self.block.find_var_block(upv) {
                        Some(blk) => {
                            blk.ref_upval = true;
                            (OP_MOVE, blk.locals.find(upv).unwrap() as i32)
                        }
                        None => {
                            match self.upval.find(upv) {
                                Some(i) => (OP_GETUPVAL, i as i32),
                                None => (OP_GETUPVAL, -1)
                            }
                        }
                    };
                    if index == -1 {
                        index = self.upval.register_unique(upv.clone()) as i32;
                    }
                    self.code.add_ABC(op, 0, index, 0, start_line(expr));
                }
            }
        };
        sused
    }

    fn compile_expr_with_propagation(&mut self, expr: &ExprNode, reg: &mut usize, save: &mut usize, loadk: bool) {
        let incr = self.compile_expr(*reg, expr, &ExprContext::with_opt(0));
        match expr.inner() {
            Expr::BinaryOp(BinaryOpr::And, _, _) | Expr::BinaryOp(BinaryOpr::Or, _, _) => {
                *save = *reg;
                *reg += incr;
            }
            _ => {
                let top = self.reg_top();
                self.code.propagate_KMV(top, save, reg, incr, loadk)
            }
        }
    }

    fn compile_expr_with_KMV_propagation(&mut self, expr: &ExprNode, reg: &mut usize, save: &mut usize) {
        self.compile_expr_with_propagation(expr, reg, save, true)
    }

    fn compile_expr_with_MV_propagation(&mut self, expr: &ExprNode, reg: &mut usize, save: &mut usize) {
        self.compile_expr_with_propagation(expr, reg, save, false)
    }

    fn compile_assign_stmt_left(&mut self, lhs: &Vec<ExprNode>) -> (usize, Vec<AssignContext>) {
        let mut reg = self.reg_top();
        let len = lhs.len();
        let mut acs = Vec::<AssignContext>::with_capacity(len);
        for (i, expr) in lhs.iter().enumerate() {
            let islast = i == len - 1;
            match expr.inner() {
                &Expr::Ident(ref s) => {
                    let identtype = self.get_ident_reftype(s);
                    let mut expr_ctx = ExprContext::new(identtype, REG_UNDEFINED, 0);
                    match identtype {
                        ExprScope::Global => {
                            self.const_index(Rc::new(Value::String(s.clone())));
                        }
                        ExprScope::Upval => {
                            self.upval.register_unique(s.clone());
                        }
                        ExprScope::Local => {
                            if islast {
                                // TODO: check
                                expr_ctx.reg = self.find_local_var(s).unwrap();
                            }
                        }
                        _ => unreachable!("invalid lhs identity type")
                    };
                    acs.push(AssignContext::new(expr_ctx, 0, 0, false, false))
                }
                &Expr::AttrGet(ref obj, ref key) => {
                    let mut expr_ctx = ExprContext::new(ExprScope::Table, REG_UNDEFINED, 0);
                    self.compile_expr_with_KMV_propagation(obj, &mut reg, &mut expr_ctx.reg);
                    let keyks = if let Expr::String(_) = key.inner() { true } else { false };
                    let mut assi_ctx = AssignContext::new(expr_ctx, 0, 0, keyks, false);
                    self.compile_expr_with_KMV_propagation(key, &mut reg, &mut assi_ctx.keyrk);
                    acs.push(assi_ctx);
                }
                _ => unreachable!("invalid left expression:{:#?}", expr.inner())
            }
        };

        (reg, acs)
    }

    fn compile_assign_stmt_right(&mut self, mut reg: usize,
                                 lhs: &Vec<ExprNode>,
                                 rhs: &Vec<ExprNode>,
                                 mut acs: Vec<AssignContext>) -> (usize, Vec<AssignContext>) {
        let lennames = lhs.len();
        let lenexprs = rhs.len();
        let mut namesassigned = 0;
        while namesassigned < lennames {
            // multiple assign with vararg function
            if namesassigned < rhs.len() && rhs[namesassigned].inner().is_vararg() && (lenexprs - namesassigned - 1) <= 0 {
                let opt = lennames - namesassigned - 1;
                let regstart = reg;
                let incr = self.compile_expr(reg, &rhs[namesassigned], &ExprContext::with_opt(opt as i32));
                reg += incr;
                for i in namesassigned..(namesassigned + incr) {
                    acs[i].nmove = true;
                    if acs[i].expr_ctx.scope == ExprScope::Table {
                        acs[i].valrk = regstart + (i - namesassigned);
                    }
                }
                namesassigned = lennames;
                break;
            }

            // regular assignment
            let ac = &mut acs[namesassigned];
            let mut nilexprs: Vec<ExprNode> = vec![];
            let expr = if namesassigned >= lenexprs {
                let expr = ExprNode::new(Expr::Nil, lhs[namesassigned].lineinfo());
                nilexprs.push(expr);
                &nilexprs[0]
            } else {
                &rhs[namesassigned]
            };

            let idx = reg;
            let incr = self.compile_expr(reg, &expr, &ac.expr_ctx);
            if ac.expr_ctx.scope == ExprScope::Table {
                match expr.inner() {
                    Expr::BinaryOp(BinaryOpr::And, _, _) | Expr::BinaryOp(BinaryOpr::Or, _, _) => {
                        ac.valrk = idx;
                        reg += incr;
                    }
                    _ => {
                        let regtop = self.reg_top();
                        self.code.propagate_KMV(regtop, &mut ac.valrk, &mut reg, incr, true);
                    }
                }
            } else {
                ac.nmove = incr != 0;
                reg += incr;
            }
            namesassigned += 1;
        }

        let rightreg = reg - 1;
        for i in namesassigned..lenexprs {
            let opt = if i != lenexprs - 1 { 0 } else { -1 };
            reg += self.compile_expr(reg, &rhs[i], &ExprContext::with_opt(opt));
        }
        (rightreg, acs)
    }

    fn compile_assign_stmt(&mut self, lhs: &Vec<ExprNode>, rhs: &Vec<ExprNode>) {
        let lhslen = lhs.len();
        let (reg, acs) = self.compile_assign_stmt_left(lhs);
        let (reg, acs) = self.compile_assign_stmt_right(reg, lhs, rhs, acs);
        let mut reg = reg as i32;

        for j in 0..lhslen {
            let i = lhslen - 1 - j;
            let expr = &lhs[i];
            match acs[i].expr_ctx.scope {
                ExprScope::Local => {
                    if acs[i].nmove {
                        if let Expr::Ident(ref s) = expr.inner() {
                            let index = match self.find_local_var(s) {
                                Some(i) => i as i32,
                                None => -1
                            };
                            self.code.add_ABC(OP_MOVE, index, reg, 0, start_line(expr));
                            reg -= 1;
                        } else {
                            unreachable!()
                        }
                    }
                }
                ExprScope::Global => {
                    if let Expr::Ident(ref s) = expr.inner() {
                        let index = self.const_index(Rc::new(Value::String(s.clone())));
                        self.code.add_ABx(OP_SETGLOBAL, reg, index as i32,  start_line(expr));
                        reg -= 1;
                    } else {
                        unreachable!()
                    }
                }
                ExprScope::Upval => {
                    if let Expr::Ident(ref s) = expr.inner() {
                        let index = self.upval.register_unique(s.clone());
                        self.code.add_ABC(OP_SETUPVAL, reg, index as i32, 0, start_line(expr));
                        reg -= 1;
                    } else {
                        unreachable!()
                    }
                }
                ExprScope::Table => {
                    let opcode = if acs[i].keyks { OP_SETTABLEKS } else { OP_SETTABLE };
                    self.code.add_ABC(opcode, acs[i].expr_ctx.reg as i32, acs[i].keyrk as i32, acs[i].valrk as i32, start_line(expr));
                    if !is_k(acs[i].valrk as i32) {
                        reg -= 1;
                    }
                }
                _ => {}
            }
        }
    }

    fn compile_reg_assignment(&mut self, names: &Vec<String>, exprs: &Vec<ExprNode>,
                              mut reg: usize, nvars: usize, line: u32) {
        let lennames = names.len();
        let lenexprs = exprs.len();

        let mut namesassigned = 0;
        let mut expr_ctx = ExprContext::with_opt(0);
        while namesassigned < lennames && namesassigned < lenexprs {
            if exprs[namesassigned].inner().is_vararg() && (lenexprs - namesassigned - 1) <= 0 {
                let opt = nvars - namesassigned;
                expr_ctx.update(ExprScope::Vararg, reg, (opt - 1) as i32);
                self.compile_expr(reg, &exprs[namesassigned], &expr_ctx);
                reg += opt;
                namesassigned = lennames;
            } else {
                expr_ctx.update(ExprScope::Local, reg, 0);
                self.compile_expr(reg, &exprs[namesassigned], &expr_ctx);
                reg += 1;
                namesassigned += 1;
            }
        }

        if lennames > namesassigned {
            let left = lennames - namesassigned - 1;
            self.code.add_ABC(OP_LOADNIL, reg as i32, (reg + left) as i32, 0, line);
            reg += left;
        }

        for i in namesassigned..lenexprs {
            let opt = if i != lenexprs - 1 { 0 } else { -1 };
            expr_ctx.update(ExprScope::None, reg, opt);
            reg += self.compile_expr(reg, &exprs[i], &expr_ctx);
        }
    }

    fn compile_local_assign_stmt(&mut self, names: &Vec<String>, values: &Vec<ExprNode>, line: u32) {
        let reg = self.reg_top();
        if names.len() == 1 && values.len() == 1 {
            if let Expr::Function(ref params, ref stmts) = values[0].inner() {
                self.register_local_var(names[0].clone());
                self.compile_reg_assignment(names, values, reg, names.len(), line);
                return;
            }
        }

        self.compile_reg_assignment(names, values, reg, names.len(), line);
        for name in names {
            self.register_local_var(name.clone());
        }
    }

    fn compile_branch_condition(&mut self, mut reg: usize, expr: &ExprNode,
                                thenlabel: i32, elselabel: i32, hasnextcond: bool) {
        let startline = start_line(expr);
        let (flip, jumplabel) = if hasnextcond { (1, thenlabel) } else { (0, elselabel) };
        match expr.inner() {
            &Expr::False | &Expr::Nil => {
                if !hasnextcond {
                    self.code.add_ASBx(OP_JMP, 0, elselabel, startline);
                }
            }
            &Expr::True | &Expr::Number(_) | &Expr::String(_) => {
                if !hasnextcond {
                    return;
                }
            }
            &Expr::UnaryOp(UnaryOpr::Not, ref ex) => {
                self.compile_branch_condition(reg, ex, elselabel, thenlabel, !hasnextcond);
                return;
            }
            &Expr::BinaryOp(BinaryOpr::And, ref lhs, ref rhs) => {
                let nextcondlabel = self.new_label();
                self.compile_branch_condition(reg, lhs, nextcondlabel, elselabel, false);
                let lastpc = self.code.last_pc();
                self.set_label_pc(nextcondlabel, lastpc);
                self.compile_branch_condition(reg, rhs, thenlabel, elselabel, hasnextcond);
                return;
            }
            &Expr::BinaryOp(BinaryOpr::Or, ref lhs, ref rhs) => {
                let nextcondlabel = self.new_label();
                self.compile_branch_condition(reg, lhs, thenlabel, nextcondlabel, true);
                let lastpc = self.code.last_pc();
                self.set_label_pc(nextcondlabel, lastpc);
                self.compile_branch_condition(reg, rhs, thenlabel, elselabel, hasnextcond);
                return;
            }
            &Expr::BinaryOp(ref opr, ref lhs, ref rhs)
            if opr == &BinaryOpr::Eq ||
                opr == &BinaryOpr::LT ||
                opr == &BinaryOpr::LE ||
                opr == &BinaryOpr::NE ||
                opr == &BinaryOpr::GT ||
                opr == &BinaryOpr::GE => {
                self.compile_binary_rel_expr_aux(reg, *opr, lhs, rhs, flip, jumplabel, startline);
                return;
            }
            _ => {}
        }

        let mut a = reg;
        self.compile_expr_with_MV_propagation(expr, &mut reg, &mut a);
        self.code.add_ABC(OP_TEST, a as i32, 0, 0 ^ flip, startline);
        self.code.add_ASBx(OP_JMP, 0, jumplabel, startline)
    }

    fn compile_while_stmt(&mut self, cond: &ExprNode, stmts: &Vec<StmtNode>,
                          star_line: u32, end_line: u32) {
        let thenlabel = self.new_label();
        let elselabel = self.new_label();
        let condlabel = self.new_label();

        let lastpc = self.code.last_pc();
        self.set_label_pc(condlabel, lastpc);
        let regtop = self.reg_top();
        self.compile_branch_condition(regtop, cond, thenlabel, elselabel, false);
        let lastpc = self.code.last_pc();
        self.set_label_pc(thenlabel, lastpc);
        self.enter_block(elselabel as usize, (star_line, end_line));
        self.compile_chunk(stmts);
        self.close_upval();
        self.code.add_ASBx(OP_JMP, 0, condlabel as i32, end_line);
        self.leave_block();
        let lastpc = self.code.last_pc();
        self.set_label_pc(elselabel, lastpc);
    }

    fn compile_repeat_stmt(&mut self, cond: &ExprNode, stmts: &Vec<StmtNode>,
                           star_line: u32, end_line: u32) {
        let initlabel = self.new_label();
        let thenlabel = self.new_label();
        let elselabel = self.new_label();

        let lastpc = self.code.last_pc();
        self.set_label_pc(initlabel, lastpc);
        self.set_label_pc(elselabel, lastpc);

        self.enter_block(thenlabel as usize, (star_line, end_line));
        self.compile_chunk(stmts);
        let regtop = self.reg_top();
        self.compile_branch_condition(regtop, cond, thenlabel, elselabel, false);

        let lastpc = self.code.last_pc();
        self.set_label_pc(thenlabel, lastpc);

        match self.leave_block() {
            Some(n) => {
                let label = self.new_label();
                self.code.add_ASBx(OP_JMP, 0, label, end_line);
                let lastpc = self.code.last_pc();
                self.set_label_pc(elselabel, lastpc);
                self.code.add_ABC(OP_CLOSE, n as i32, 0, 0, end_line);
                self.code.add_ASBx(OP_JMP, 0, initlabel, end_line);
                let lastpc = self.code.last_pc();
                self.set_label_pc(label, lastpc);
            }
            None => {}
        }
    }

    fn compile_if_stmt(&mut self, ifelsethen: &IfThenElse, startline: u32, endline: u32) {
        let thenlabel = self.new_label();
        let elselabel = self.new_label();
        let endlabel = self.new_label();

        let regtop = self.reg_top();
        self.compile_branch_condition(regtop, &ifelsethen.condition, thenlabel, elselabel, false);
        let lastpc = self.code.last_pc();
        self.set_label_pc(thenlabel, lastpc);
        self.compile_block(&ifelsethen.then);
        if ifelsethen.els.len() > 0 {
            self.code.add_ASBx(OP_JMP, 0, endlabel, startline)
        }

        let lastpc = self.code.last_pc();
        self.set_label_pc(elselabel, lastpc);
        if ifelsethen.els.len() > 0 {
            self.compile_block(&ifelsethen.els);
            let lastpc = self.code.last_pc();
            self.set_label_pc(endlabel, lastpc);
        }
    }

    fn compile_nfor_stmt(&mut self, nfor: &NumberFor, startline: u32, endline: u32) {
        let endlabel = self.new_label();
        let mut expr_ctx = ExprContext::with_opt(0);

        self.enter_block(endlabel as usize, (startline, endline));
        let regtop = self.reg_top();
        let rindex = self.register_local_var(String::from("(for index)"));
        expr_ctx.update(ExprScope::Local, rindex, 0);
        self.compile_expr(regtop, &nfor.init, &expr_ctx);

        let regtop = self.reg_top();
        let rlimit = self.register_local_var(String::from("(for limit)"));
        expr_ctx.update(ExprScope::Local, rlimit, 0);
        self.compile_expr(regtop, &nfor.limit, &expr_ctx);

        let regtop = self.reg_top();
        let rstep = self.register_local_var(String::from("(for step)"));
        expr_ctx.update(ExprScope::Local, rstep, 0);
        self.compile_expr(regtop, &nfor.step, &expr_ctx);

        self.code.add_ASBx(OP_FORPREP, rindex as i32, 0, startline);
        self.register_local_var(nfor.name.clone());

        let bodypc = self.code.last_pc();
        self.compile_chunk(&nfor.stmts);
        self.leave_block();
        let flpc = self.code.last_pc();
        self.code.add_ASBx(OP_FORLOOP, rindex as i32, bodypc as i32 - (flpc as i32 + 1), startline);

        let lastpc = self.code.last_pc();
        self.set_label_pc(endlabel, lastpc);
        self.code.set_argsbx(bodypc, (flpc - bodypc) as i32);
    }

    fn compile_gfor_stmt(&mut self, gfor: &GenericFor, startline: u32, endline: u32) {
        let endlabel = self.new_label();
        let bodylable = self.new_label();
        let fllabel = self.new_label();

        let nnames = gfor.names.len();
        self.enter_block(endlabel as usize, (startline, endline));
        let rgen = self.register_local_var(String::from("(for generator)"));
        self.register_local_var(String::from("(for state)"));
        self.register_local_var(String::from("(for control)"));

        let regtop = self.reg_top();
        self.compile_reg_assignment(&gfor.names, &gfor.exprs, regtop - 3, 3, startline);

        self.code.add_ASBx(OP_JMP, 0, fllabel, startline);
        for name in &gfor.names {
            self.register_local_var(name.clone());
        }

        let lastpc = self.code.last_pc();
        self.set_label_pc(bodylable, lastpc);
        self.compile_chunk(&gfor.stmts);
        self.leave_block();

        let lastpc = self.code.last_pc();
        self.set_label_pc(fllabel, lastpc);
        self.code.add_ABC(OP_TFORLOOP, rgen as i32, 0, nnames as i32, startline);
        self.code.add_ASBx(OP_JMP, 0, bodylable as i32, startline);

        let lastpc = self.code.last_pc();
        self.set_label_pc(endlabel, lastpc);
    }

    fn compile_return_stmt(&mut self, exprs: &Vec<ExprNode>, startline: u32, endline: u32) {
        let lenexprs = exprs.len();
        let mut reg = self.reg_top();
        let a = reg;
        let mut lastisvararg = false;

        if lenexprs == 1 {
            match exprs[0].inner() {
                Expr::Ident(ref s) => {
                    if let Some(index) = self.find_local_var(s) {
                        self.code.add_ABC(OP_RETURN, index as i32, 2, 0, startline);
                        return;
                    }
                }
                Expr::FuncCall(ref expr) => {
                    //reg += self.compile_expr(reg, &exprs[0], &ExprContext::with_opt(-2));
                    let lastpc = self.code.last_pc();
                    self.code.set_opcode(lastpc, OP_TAILCALL);
                    self.code.add_ABC(OP_RETURN, a as i32, 0, 0, startline);
                    return;
                }
                _ => {}
            }
        }

        for (i, expr) in exprs.iter().enumerate() {
            if i == lenexprs - 1 && expr.inner().is_vararg() {
                self.compile_expr(reg, expr, &ExprContext::with_opt(-2));
                lastisvararg = true;
            } else {
                reg += self.compile_expr(reg, expr, &ExprContext::with_opt(0))
            }
        }

        let count = if lastisvararg { 0 } else { reg - a + 1 };
        self.code.add_ABC(OP_RETURN, a as i32, count as i32, 0, startline);
    }

    fn compile_break_stmt(&mut self, startline: u32) {
        let mut blk = &self.block;
        loop {
            let label = blk.break_label;
            if label != LABEL_NO_JUMP {
                if blk.ref_upval {
                    match blk.parent {
                        Some(ref parent) => {
                            self.code.add_ABC(OP_CLOSE, parent.locals.last_index() as i32, 0, 0, startline);
                        }
                        None => unreachable!()
                    }
                }
                self.code.add_ASBx(OP_JMP, 0, label as i32, startline);
                return;
            }
            match blk.parent {
                Some(ref parent) => blk = parent,
                None => break
            }
        }
        panic!("no loop to break: {}", startline);
    }

    fn compile_stmt(&mut self, stmt: &StmtNode) {
        let (startline, endline) = (start_line(stmt), end_line(stmt));
        match stmt.inner() {
            &Stmt::Assign(ref lhs, ref rhs) => self.compile_assign_stmt(lhs, rhs),
            &Stmt::LocalAssign(ref names, ref values) => self.compile_local_assign_stmt(names, values, startline),
            &Stmt::FuncCall(ref expr) | &Stmt::MethodCall(ref expr) => {
                let regtop = self.reg_top();
                self.compile_fncall_expr(regtop, expr, &ExprContext::with_opt(-1));
            }
            &Stmt::DoBlock(ref stmts) => {
                self.enter_block(LABEL_NO_JUMP, (startline, endline));
                self.compile_chunk(stmts);
                self.leave_block();
            }
            &Stmt::While(ref cond, ref stmts) => self.compile_while_stmt(cond, stmts, startline, endline),
            &Stmt::Repeat(ref cond, ref stmts) => self.compile_repeat_stmt(cond, stmts, startline, endline),
            &Stmt::If(ref ifthenelse) => self.compile_if_stmt(ifthenelse, startline, endline),
            &Stmt::NumberFor(ref nfor) => self.compile_nfor_stmt(nfor, startline, endline),
            &Stmt::GenericFor(ref gfor) => self.compile_gfor_stmt(gfor, startline, endline),
            &Stmt::FuncDef(ref funcdef) => self.compile_assign_stmt(&funcdef.name, &funcdef.body),
            &Stmt::MethodDef(ref methoddef) => {
                let mut regtop = self.reg_top();
                let mut treg = 0;
                self.compile_expr_with_KMV_propagation(&methoddef.receiver, &mut regtop, &mut treg);
                let kreg = self.load_rk(&mut regtop, &methoddef.body, Rc::new(Value::String(methoddef.method.clone())));
                self.compile_expr(regtop, &methoddef.body, &ExprContext::new(ExprScope::Method, REG_UNDEFINED, 0));
                self.code.add_ABC(OP_SETTABLE, treg as i32, kreg as i32, regtop as i32, start_line(&methoddef.receiver))
            }
            &Stmt::Return(ref exprs) => self.compile_return_stmt(exprs, startline, endline),
            &Stmt::Break => self.compile_break_stmt(startline),
        }
    }

    fn patchcode(&mut self) {
        let mut maxreg = if self.proto.param_count > 1 { self.proto.param_count as i32 } else { 1 };
        let mut moven = 0;
        let mut pc = 0;
        let lastpc = self.code.pc;
        while pc < lastpc {
            let inst = self.code.at(pc);
            let curop = get_opcode(inst);
            match curop {
                OP_CLOSURE => {
                    pc += self.proto.prototypes[get_argbx(inst) as usize].upval_count as usize + 1;
                    moven = 0;
                    continue;
                }
                OP_SETGLOBAL | OP_SETUPVAL | OP_EQ | OP_LT | OP_LE | OP_TEST |
                OP_TAILCALL | OP_RETURN | OP_FORPREP | OP_FORLOOP | OP_TFORLOOP |
                OP_SETLIST | OP_CLOSE => {}
                OP_CALL => {
                    let reg = get_arga(inst) + get_argb(inst) - 2;
                    if reg > maxreg {
                        maxreg = reg;
                    }
                }
                OP_VARARG => {
                    let reg = get_arga(inst) + get_argb(inst) - 1;
                    if reg > maxreg {
                        maxreg = reg;
                    }
                }
                OP_SELF => {
                    let reg = get_arga(inst) + 1;
                    if reg > maxreg {
                        maxreg = reg;
                    }
                }
                OP_LOADNIL => {
                    let reg = get_argb(inst);
                    if reg > maxreg {
                        maxreg = reg;
                    }
                }
                OP_JMP => {
                    let mut distance = 0;
                    let mut count = 0;
                    let mut jmp = inst;
                    while get_opcode(jmp) == OP_JMP && count < 5 {
                        let d = self.get_label_pc(get_argsbx(jmp)) as i32 - pc as i32;
                        if d > OPCODE_MAXSBx {
                            if distance == 0 {
                                panic!("too long to jump")
                            }
                            break;
                        }
                        distance = d;
                        count += 1;
                        jmp = self.code.at((pc as i32 + distance + 1) as usize);
                    }

                    if distance == 0 {
                        self.code.set_opcode(pc, OP_NOP);
                    } else {
                        self.code.set_argsbx(pc, distance as i32);
                    }
                }
                _ => {
                    let reg = get_arga(inst);
                    if reg > maxreg {
                        maxreg = reg;
                    }
                }
            }
            if curop == OP_MOVE {
                moven += 1;
            } else {
                if moven > 1 {
                    self.code.set_opcode(pc - moven, OP_MOVEN);
                    let c = if moven - 1 < OPCODE_MAXC as usize { (moven - 1) as i32 } else { OPCODE_MAXC };
                    self.code.set_argc(pc - moven, c);
                }
                moven = 0;
            }

            pc += 1;
        }

        maxreg += 1;
        if maxreg > MAX_REGISTERS {
            panic!("<{}:{:?}> register overflow(too many local variables)", self.proto.source, self.proto.lineinfo)
        }
        self.proto.used_registers = maxreg as u8;
    }

    fn compile_func_expr(&mut self, params: &ParList, stmts: &Vec<StmtNode>,
                         expr_ctx: &ExprContext, lineinfo: (u32, u32)) {
        self.proto.lineinfo = lineinfo;
        let endline = lineinfo.1;
        if params.names.len() > (MAX_REGISTERS as usize) {
            panic!("register overflow")
        }
        self.proto.param_count = params.names.len() as u8;
        if expr_ctx.scope == ExprScope::Method {
            self.proto.param_count += 1;
            self.register_local_var(String::from("self"));
        }
        for name in &params.names {
            self.register_local_var(name.clone());
        }

        if params.vargs {
            // compact vararg
            self.proto.is_vararg = VARARG_HAS | VARARG_NEED;
            if let Some(_) = self.parent {
                self.register_local_var(String::from("arg"));
            }
            self.proto.is_vararg |= VARARG_IS;
        }

        self.compile_chunk(stmts);

        self.code.add_ABC(OP_RETURN, 0, 1, 0, endline);
        self.end_scope();
        let codestore = Instructions::new();

        self.proto.code = self.code.list();
        self.proto.debug_pos = self.code.line_list();
        self.proto.debug_upval = self.upval.names();
        self.proto.upval_count = self.proto.debug_upval.len() as u8;
        let mut strv: Vec<String> = vec![];
        for clv in &self.proto.constants {
            let sv = if let Value::String(ref s) = **clv { s.clone() } else { String::new() };
            strv.push(sv);
        }
        self.proto.strings = strv;
        self.patchcode();
    }
}

pub fn compile(stmts: Vec<StmtNode>, name: String) -> Result<Box<FunctionProto>> {
    let mut compiler = Compiler::new(name, None);
    let mut par = ParList::new();
    par.set_vargs(true);
    let lineinfo = if stmts.len() > 0 { (start_line(&stmts[0]), end_line(&stmts[stmts.len() - 1])) } else { (1, 1) };
    compiler.compile_func_expr(&par, &stmts, &ExprContext::with_opt(0), lineinfo);
    Ok(compiler.proto)
}
