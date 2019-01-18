#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::prelude::v1::*;
pub const INVALID_INSTRUCTION: u32 = 0xFFFFFFFF;

type OpCodeSize = i32;

pub const OPCODE_SIZE: OpCodeSize = 6;
pub const OPCODE_SIZEA: OpCodeSize = 9;
pub const OPCODE_SIZEB: OpCodeSize = 9;
pub const OPCODE_SIZEC: OpCodeSize = 9;
pub const OPCODE_SIZEBx: OpCodeSize = 18;
pub const OPCODE_SIZESBx: OpCodeSize = 18;

pub const OPCODE_MAXA: OpCodeSize = 1 << OPCODE_SIZEA - 1;
pub const OPCODE_MAXB: OpCodeSize = 1 << OPCODE_SIZEB - 1;
pub const OPCODE_MAXC: OpCodeSize = 1 << OPCODE_SIZEC - 1;
pub const OPCODE_MAXBx: OpCodeSize = 1 << OPCODE_SIZEBx - 1;
pub const OPCODE_MAXSBx: OpCodeSize = OPCODE_MAXBx >> 1;

/// Lua 5.1.4 opcodes layout:
///
/// instruction = 32bit(fixed length)
///
/// +---------------------------------------------+
/// |0-5(6bits)|6-13(8bit)|14-22(9bit)|23-31(9bit)|
/// |==========+==========+===========+===========|
/// |  opcode  |    A     |     C     |    B      |
/// |----------+----------+-----------+-----------|
/// |  opcode  |    A     |      Bx(unsigned)     |
/// |----------+----------+-----------+-----------|
/// |  opcode  |    A     |      sBx(signed)      |
/// +---------------------------------------------+

pub type OpCode = i32;

/// A B     R(A) := R(B)
pub const OP_MOVE: OpCode = 0;
/// A B     R(A) := R(B); followed by R(C) MOVE ops
pub const OP_MOVEN: OpCode = 1;
/// A Bx    R(A) := Kst(Bx)
pub const OP_LOADK: OpCode = 2;
/// A B C   R(A) := (Bool)B; if (C) pc++
pub const OP_LOADBOOL: OpCode = 3;
/// A B     R(A) := ... := R(B) := nil
pub const OP_LOADNIL: OpCode = 4;
/// A B     R(A) := UpValue[B]
pub const OP_GETUPVAL: OpCode = 5;
/// A Bx    R(A) := Gbl[Kst(Bx)]
pub const OP_GETGLOBAL: OpCode = 6;
/// A B C   R(A) := R(B)[RK(C)]
pub const OP_GETTABLE: OpCode = 7;
/// A B C   R(A) := R(B)[RK(C)] ; RK(C) is pub constant string
pub const OP_GETTABLEKS: OpCode = 8;
/// A Bx    Gbl[Kst(Bx)] := R(A)
pub const OP_SETGLOBAL: OpCode = 9;
/// A B     UpValue[B] := R(A)
pub const OP_SETUPVAL: OpCode = 10;
/// A B C   R(A)[RK(B)] := RK(C)
pub const OP_SETTABLE: OpCode = 11;
/// A B C   R(A)[RK(B)] := RK(C) ; RK(B) is pub constant string
pub const OP_SETTABLEKS: OpCode = 12;
/// A B C   R(A) := {} (size = BC)
pub const OP_NEWTABLE: OpCode = 13;
/// A B C   R(A+1) := R(B); R(A) := R(B)[RK(C)]
pub const OP_SELF: OpCode = 14;
/// A B C   R(A) := RK(B) + RK(C)
pub const OP_ADD: OpCode = 15;
/// A B C   R(A) := RK(B) - RK(C)
pub const OP_SUB: OpCode = 16;
/// A B C   R(A) := RK(B) * RK(C)
pub const OP_MUL: OpCode = 17;
/// A B C   R(A) := RK(B) / RK(C)
pub const OP_DIV: OpCode = 18;
/// A B C   R(A) := RK(B) % RK(C)
pub const OP_MOD: OpCode = 19;
/// A B C   R(A) := RK(B) ^ RK(C)
pub const OP_POW: OpCode = 20;
/// A B     R(A) := -R(B)
pub const OP_UNM: OpCode = 21;
/// A B     R(A) := not R(B)
pub const OP_NOT: OpCode = 22;
/// A B     R(A) := length of R(B)
pub const OP_LEN: OpCode = 23;
/// A B C   R(A) := R(B).. ... ..R(C)
pub const OP_CONCAT: OpCode = 24;
/// sBx     pc+=sBx
pub const OP_JMP: OpCode = 25;
/// A B C   if ((RK(B) == RK(C)) ~= A) then pc++
pub const OP_EQ: OpCode = 26;
/// A B C   if ((RK(B) <  RK(C)) ~= A) then pc++
pub const OP_LT: OpCode = 27;
/// A B C   if ((RK(B) <= RK(C)) ~= A) then pc++
pub const OP_LE: OpCode = 28;
/// A C     if not (R(A) <=> C) then pc++
pub const OP_TEST: OpCode = 29;
/// A B C   if (R(B) <=> C) then R(A) := R(B) else pc++
pub const OP_TESTSET: OpCode = 30;
/// A B C   R(A) ... R(A+C-2) := R(A)(R(A+1) ... R(A+B-1))
pub const OP_CALL: OpCode = 31;
/// A B C   return R(A)(R(A+1) ... R(A+B-1))
pub const OP_TAILCALL: OpCode = 32;
/// A B     return R(A) ... R(A+B-2)      (see note)
pub const OP_RETURN: OpCode = 33;
/// A sBx   R(A)+=R(A+2);
///         if R(A) <?= R(A+1) then { pc+=sBx; R(A+3)=R(A) }
pub const OP_FORLOOP: OpCode = 34;
/// A sBx   R(A)-=R(A+2); pc+=sBx
pub const OP_FORPREP: OpCode = 35;
/// A C     R(A+3) ... R(A+3+C) := R(A)(R(A+1) R(A+2));
///         if R(A+3) ~= nil then { pc++; R(A+2)=R(A+3); }
pub const OP_TFORLOOP: OpCode = 36;
/// A B C   R(A)[(C-1)*FPF+i] := R(A+i) 1 <= i <= B
pub const OP_SETLIST: OpCode = 37;
/// A       close all variables in the stack up to (>=) R(A)
pub const OP_CLOSE: OpCode = 38;
/// A Bx    R(A) := closure(KPROTO[Bx] R(A) ... R(A+n))
pub const OP_CLOSURE: OpCode = 39;
/// A B     R(A) R(A+1) ... R(A+B-1) = vararg
pub const OP_VARARG: OpCode = 40;
/// NOP
pub const OP_NOP: OpCode = 41;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum OpArgMode {
    N,
    U,
    R,
    K,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum OpType {
    ABC,
    ABx,
    ASBx,
}

impl ToString for OpType {
    fn to_string(&self) -> String {
        match self {
            &OpType::ABC => "ABC".to_string(),
            &OpType::ABx => "ABx".to_string(),
            &OpType::ASBx => "ASBx".to_string(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct OpProp {
    name: &'static str,
    is_test: bool,
    set_reg_a: bool,
    mode_arg_b: OpArgMode,
    mode_arg_c: OpArgMode,
    typ: OpType,
}

static OP_NAMES: &'static [OpProp; (OP_NOP as usize + 1)] = &[
    OpProp { name: "MOVE", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::N, typ: OpType::ABC },
    OpProp { name: "MOVEN", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::N, typ: OpType::ABC },
    OpProp { name: "LOADK", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::N, typ: OpType::ABx },
    OpProp { name: "LOADBOOL", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::U, mode_arg_c: OpArgMode::U, typ: OpType::ABC },
    OpProp { name: "LOADNIL", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::N, typ: OpType::ABC },
    OpProp { name: "GETUPVAL", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::U, mode_arg_c: OpArgMode::N, typ: OpType::ABC },
    OpProp { name: "GETGLOBAL", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::N, typ: OpType::ABx },
    OpProp { name: "GETTABLE", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "GETTABLEKS", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "SETGLOBAL", is_test: false, set_reg_a: false, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::N, typ: OpType::ABx },
    OpProp { name: "SETUPVAL", is_test: false, set_reg_a: false, mode_arg_b: OpArgMode::U, mode_arg_c: OpArgMode::N, typ: OpType::ABC },
    OpProp { name: "SETTABLE", is_test: false, set_reg_a: false, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "SETTABLEKS", is_test: false, set_reg_a: false, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "NEWTABLE", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::U, mode_arg_c: OpArgMode::U, typ: OpType::ABC },
    OpProp { name: "SELF", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "ADD", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "SUB", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "MUL", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "DIV", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "MOD", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "POW", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "UNM", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::N, typ: OpType::ABC },
    OpProp { name: "NOT", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::N, typ: OpType::ABC },
    OpProp { name: "LEN", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::N, typ: OpType::ABC },
    OpProp { name: "CONCAT", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::R, typ: OpType::ABC },
    OpProp { name: "JMP", is_test: false, set_reg_a: false, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::N, typ: OpType::ASBx },
    OpProp { name: "EQ", is_test: true, set_reg_a: false, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "LT", is_test: true, set_reg_a: false, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "LE", is_test: true, set_reg_a: false, mode_arg_b: OpArgMode::K, mode_arg_c: OpArgMode::K, typ: OpType::ABC },
    OpProp { name: "TEST", is_test: true, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::U, typ: OpType::ABC },
    OpProp { name: "TESTSET", is_test: true, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::U, typ: OpType::ABC },
    OpProp { name: "CALL", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::U, mode_arg_c: OpArgMode::U, typ: OpType::ABC },
    OpProp { name: "TAILCALL", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::U, mode_arg_c: OpArgMode::U, typ: OpType::ABC },
    OpProp { name: "RETURN", is_test: false, set_reg_a: false, mode_arg_b: OpArgMode::U, mode_arg_c: OpArgMode::N, typ: OpType::ABC },
    OpProp { name: "FORLOOP", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::N, typ: OpType::ASBx },
    OpProp { name: "FORPREP", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::N, typ: OpType::ASBx },
    OpProp { name: "TFORLOOP", is_test: true, set_reg_a: false, mode_arg_b: OpArgMode::N, mode_arg_c: OpArgMode::U, typ: OpType::ABC },
    OpProp { name: "SETLIST", is_test: false, set_reg_a: false, mode_arg_b: OpArgMode::U, mode_arg_c: OpArgMode::U, typ: OpType::ABC },
    OpProp { name: "CLOSE", is_test: false, set_reg_a: false, mode_arg_b: OpArgMode::N, mode_arg_c: OpArgMode::N, typ: OpType::ABC },
    OpProp { name: "CLOSURE", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::U, mode_arg_c: OpArgMode::N, typ: OpType::ABx },
    OpProp { name: "VARARG", is_test: false, set_reg_a: true, mode_arg_b: OpArgMode::U, mode_arg_c: OpArgMode::N, typ: OpType::ABC },
    OpProp { name: "NOP", is_test: false, set_reg_a: false, mode_arg_b: OpArgMode::R, mode_arg_c: OpArgMode::N, typ: OpType::ASBx },
];

pub fn op_to_string(op: i32) -> String {
    OP_NAMES[op as usize].name.to_string()
}

pub type Instruction = u32;

#[inline]
pub fn get_opcode(inst: Instruction) -> OpCode {
    (inst >> 26) as OpCode
}

#[inline]
pub fn set_opcode(inst: &mut Instruction, op: OpCode) {
    *inst = (*inst & 0x3ffffff) | (op as u32) << 26
}

#[inline]
pub fn get_arga(inst: Instruction) -> i32 {
    ((inst >> 18) & 0xff) as i32
}

#[inline]
pub fn set_arga(inst: &mut Instruction, a: i32) {
    *inst = (*inst & 0xfc03ffff) | ((a & 0xff) as u32) << 18
}

#[inline]
pub fn get_argb(inst: Instruction) -> i32 {
    (inst & 0x1ff) as i32
}

#[inline]
pub fn set_argb(inst: &mut Instruction, b: i32) {
    *inst = (*inst & 0xfffffe00) | ((b as u32) & 0x1ff)
}

#[inline]
pub fn get_argc(inst: Instruction) -> i32 {
    ((inst >> 9) & 0x1ff) as i32
}

#[inline]
pub fn set_argc(inst: &mut Instruction, c: i32) {
    *inst = (*inst & 0xfffc01ff) | ((c & 0x1ff) as u32) << 9
}

#[inline]
pub fn get_argbx(inst: Instruction) -> i32 {
    (inst & 0x3ffff) as i32
}

#[inline]
pub fn set_argbx(inst: &mut Instruction, bx: i32) {
    *inst = (*inst & 0xfffc0000) | (bx & 0x3ffff) as u32
}

#[inline]
pub fn get_argsbx(inst: Instruction) -> i32 {
    get_argbx(inst) - OPCODE_MAXSBx
}

#[inline]
pub fn set_argsbx(inst: &mut Instruction, sbx: i32) {
    set_argbx(inst, sbx + OPCODE_MAXSBx)
}

pub fn ABC(op: OpCode, a: i32, b: i32, c: i32) -> Instruction {
    let mut inst: Instruction = 0;
    set_opcode(&mut inst, op);
    set_arga(&mut inst, a);
    set_argb(&mut inst, b);
    set_argc(&mut inst, c);
    inst
}

pub fn ABx(op: OpCode, a: i32, bx: i32) -> Instruction {
    let mut inst: Instruction = 0;
    set_opcode(&mut inst, op);
    set_arga(&mut inst, a);
    set_argbx(&mut inst, bx);
    inst
}

pub fn ASBx(op: OpCode, a: i32, sbx: i32) -> Instruction {
    let mut inst: Instruction = 0;
    set_opcode(&mut inst, op);
    set_arga(&mut inst, a);
    set_argsbx(&mut inst, sbx);
    inst
}

pub const opBitRk: OpCodeSize = 1 << (OPCODE_SIZEB - 1);
pub const opMaxIndexRk: OpCodeSize = opBitRk - 1;

#[inline]
pub fn is_k(v: i32) -> bool {
    v & opBitRk != 0
}

#[inline]
pub fn rk_ask(v: i32) -> i32 {
    v | opBitRk
}

pub fn to_string(inst: Instruction) -> String {
    let op = get_opcode(inst);
    if op > OP_NOP {
        return String::new();
    }

    let prop = OP_NAMES[op as usize];
    let arga = get_arga(inst);
    let argb = get_argb(inst);
    let argc = get_argc(inst);
    let argbx = get_argbx(inst);
    let argsbx = get_argsbx(inst);

    let ops = match prop.typ {
        OpType::ABC => format!("{:10}| {:4} | {:4}, {:4}, {:4}", prop.name, prop.typ.to_string(), arga, argb, argc),
        OpType::ABx => format!("{:10}| {:4} | {:4}, {:4}", prop.name, prop.typ.to_string(), arga, argbx),
        OpType::ASBx => format!("{:10}| {:4} | {:4}, {:4}", prop.name, prop.typ.to_string(), arga, argsbx)
    };

    let ops = format!("{:36}", ops);

    match op {
        OP_MOVE => format!("{} | R({}) := R({})", ops, arga, argb),
        OP_MOVEN => format!("{} | R({}) := R({}); followed by {} MOVE ops", ops, arga, argb, argc),
        OP_LOADK => format!("{} | R({}) := Kst({})", ops, arga, argbx),
        OP_LOADBOOL => format!("{} | R({}) := (Bool){}; if ({}) pc++", ops, arga, argb, argc),
        OP_LOADNIL => format!("{} | R({}) := ... := R({}) := nil", ops, arga, argb),
        OP_GETUPVAL => format!("{} | R({}) := UpValue[{}]", ops, arga, argb),
        OP_GETGLOBAL => format!("{} | R({}) := Gbl[Kst({})]", ops, arga, argbx),
        OP_GETTABLE => format!("{} | R({}) := R({})[RK({})]", ops, arga, argb, argc),
        OP_GETTABLEKS => format!("{} | R({}) := R({})[RK({})] ; RK({}) is constant string", ops, arga, argb, argc, argc),
        OP_SETGLOBAL => format!("{} | Gbl[Kst({})] := R({})", ops, argbx, arga),
        OP_SETUPVAL => format!("{} | UpValue[{}] := R({})", ops, argb, arga),
        OP_SETTABLE => format!("{} | R({})[RK({})] := RK({})", ops, arga, argb, argc),
        OP_SETTABLEKS => format!("{} | R({})[RK({})] := RK({}) ; RK({}) is constant string", ops, arga, argb, argc, argb),
        OP_NEWTABLE => format!("{} | R({}) := {{}} (size = BC)", ops, arga),
        OP_SELF => format!("{} | R({}+1) := R({}); R({}) := R({})[RK({})]", ops, arga, argb, arga, argb, argc),
        OP_ADD => format!("{} | R({}) := RK({}) + RK({})", ops, arga, argb, argc),
        OP_SUB => format!("{} | R({}) := RK({}) - RK({})", ops, arga, argb, argc),
        OP_MUL => format!("{} | R({}) := RK({}) * RK({})", ops, arga, argb, argc),
        OP_DIV => format!("{} | R({}) := RK({}) / RK({})", ops, arga, argb, argc),
        OP_MOD => format!("{} | R({}) := RK({}) %% RK({})", ops, arga, argb, argc),
        OP_POW => format!("{} | R({}) := RK({}) ^ RK({})", ops, arga, argb, argc),
        OP_UNM => format!("{} | R({}) := -R({})", ops, arga, argb),
        OP_NOT => format!("{} | R({}) := not R({})", ops, arga, argb),
        OP_LEN => format!("{} | R({}) := length of R({})", ops, arga, argb),
        OP_CONCAT => format!("{} | R({}) := R({}).. ... ..R({})", ops, arga, argb, argc),
        OP_JMP => format!("{} | pc+={}", ops, argsbx),
        OP_EQ => format!("{} | if ((RK({}) == RK({})) ~= {}) then pc++", ops, argb, argc, arga),
        OP_LT => format!("{} | if ((RK({}) <  RK({})) ~= {}) then pc++", ops, argb, argc, arga),
        OP_LE => format!("{} | if ((RK({}) <= RK({})) ~= {}) then pc++", ops, argb, argc, arga),
        OP_TEST => format!("{} | if not (R({}) <=> {}) then pc++", ops, arga, argc),
        OP_TESTSET => format!("{} | if (R({}) <=> {}) then R({}) := R({}) else pc++", ops, argb, argc, arga, argb),
        OP_CALL => format!("{} | R({}) ... R({}+{}-2) := R({})(R({}+1) ... R({}+{}-1))", ops, arga, arga, argc, arga, arga, arga, argb),
        OP_TAILCALL => format!("{} | return R({})(R({}+1) ... R({}+{}-1))", ops, arga, arga, arga, argb),
        OP_RETURN => format!("{} | return R({}) ... R({}+{}-2)", ops, arga, arga, argb),
        OP_FORLOOP => format!("{} | R({})+=R({}+2); if R({}) <?= R({}+1) then {{ pc+={}; R({}+3)=R({}) }}", ops, arga, arga, arga, arga, argsbx, arga, arga),
        OP_FORPREP => format!("{} | R({})-=R({}+2); pc+={}", ops, arga, arga, argsbx),
        OP_TFORLOOP => format!("{} | R({}+3) ... R({}+3+{}) := R({})(R({}+1) R({}+2)); if R({}+3) ~= nil then {{ pc++; R({}+2)=R({}+3); }}", ops, arga, arga, argc, arga, arga, arga, arga, arga, arga),
        OP_SETLIST => format!("{} | R({})[({}-1)*FPF+i] := R({}+i) 1 <= i <= {}", ops, arga, argc, arga, argb),
        OP_CLOSE => format!("{} | close all variables in the stack up to (>=) R({})", ops, arga),
        OP_CLOSURE => format!("{} | R({}) := closure(KPROTO[{}] R({}) ... R({}+n))", ops, arga, argbx, arga, arga),
        OP_VARARG => format!("{} |  R({}) R({}+1) ... R({}+{}-1) = vararg", ops, arga, arga, arga, argb),
        OP_NOP => format!("{}", ops),
        _ => unreachable!()
    }
}
