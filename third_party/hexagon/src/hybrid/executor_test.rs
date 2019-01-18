use super::executor::Executor;
use super::function::Function;
use super::basic_block::BasicBlock;
use super::opcode::OpCode;
use super::page_table::PageTable;
use super::program::{Program, NativeFunction};
use super::program_context::ProgramContext;
use super::jit::NoJit;

#[test]
fn test_sum() {
    const END: u64 = 100000;

    let mut pt = PageTable::new();
    pt.virtual_alloc(0x08000000);
    pt.write_u64(0x08000000, 0); // exclusive
    pt.write_u64(0x08000008, END); // inclusive

    let sum_fn = Function::from_basic_blocks(vec![
        BasicBlock::from_opcodes(vec![
            { OpCode::UIConst64(0, 0x08000000) },
            { OpCode::Load64(1, 0) },
            { OpCode::UIConst64(0, 0x08000008) },
            { OpCode::Load64(2, 0) },
            { OpCode::Branch(1) }
        ]),
        BasicBlock::from_opcodes(vec![
            { OpCode::UIGe(1, 2) },
            { OpCode::ConditionalBranch(3, 2)}
        ]),
        BasicBlock::from_opcodes(vec![
            { OpCode::UIConst64(0, 1) },
            { OpCode::UIAdd(0, 1) },
            { OpCode::Mov(1, 0) },
            { OpCode::UIAdd(1, 3) },
            { OpCode::Mov(3, 0) },
            { OpCode::Branch(1) }
        ]),
        BasicBlock::from_opcodes(vec![
            { OpCode::UIConst64(0, 0x08000016) },
            { OpCode::Store64(3, 0) },
            { OpCode::Return }
        ])
    ]);

    let executor = Executor::with_page_table(pt.clone());
    let program = Program::from_functions(vec! [
        sum_fn
    ]);
    executor.eval_program(&ProgramContext::new(&executor, program, None as Option<NoJit>), 0);

    let result = pt.read_u64(0x08000016).unwrap();
    assert_eq!(result, (1 + END) * END / 2);
}

#[test]
fn test_int_types() {
    let mut pt = PageTable::new();
    pt.virtual_alloc(0x08000000);

    let test_fn = Function::from_basic_blocks(vec! [
        BasicBlock::from_opcodes(vec! [
            { OpCode::SIConst64(1, -10) },
            { OpCode::SIConst64(2, 3) },
            { OpCode::SIAdd(1, 2) },
            { OpCode::SIMul(0, 2) },
            { OpCode::UIConst64(1, 0x08000000) },
            { OpCode::Store64(0, 1) },
            { OpCode::Return }
        ])
    ]);

    let executor = Executor::with_page_table(pt.clone());
    let program = Program::from_functions(vec! [
        test_fn
    ]);
    executor.eval_program(&ProgramContext::new(&executor, program, None as Option<NoJit>), 0);

    let result = pt.read_i64(0x08000000);
    assert_eq!(result, Some((-10 + 3) * 3));
}

#[test]
fn test_fp() {
    let mut pt = PageTable::new();
    pt.virtual_alloc(0x08000000);

    let test_fn = Function::from_basic_blocks(vec! [
        BasicBlock::from_opcodes(vec! [
            { OpCode::FConst64(1, ::std::f64::consts::PI) },
            { OpCode::FConst64(2, 2.0) },
            { OpCode::FMul(1, 2) },
            { OpCode::Mov(3, 0) },
            { OpCode::FConst64(0, 1.0) },
            { OpCode::FAdd(3, 0) },
            { OpCode::Mov(3, 0) },
            { OpCode::FConst64(0, 3.0) },
            { OpCode::FSub(3, 0) },
            { OpCode::Mov(3, 0) },
            { OpCode::FConst64(0, 0.7) },
            { OpCode::FDiv(3, 0) },
            { OpCode::Mov(3, 0) },
            { OpCode::UIConst64(0, 0x08000000) },
            { OpCode::Store64(3, 0)},
            { OpCode::Return }
        ])
    ]);

    let executor = Executor::with_page_table(pt.clone());
    let program = Program::from_functions(vec! [
        test_fn
    ]);
    executor.eval_program(&ProgramContext::new(&executor, program, None as Option<NoJit>), 0);

    let result = pt.read_f64(0x08000000).unwrap();
    assert!((result - (::std::f64::consts::PI * 2.0 + 1.0 - 3.0) / 0.7).abs() < 1e-12);
}

#[test]
fn test_fn_call() {
    use std::cell::RefCell;

    let result: RefCell<u64> = RefCell::new(0);

    let setter = |executor: &Executor| -> () {
        *result.borrow_mut() = executor.read_global(1);
    };

    let mut program = Program::from_functions(vec! [
        Function::from_basic_blocks(vec! [
            BasicBlock::from_opcodes(vec! [
                { OpCode::UIConst64(0, 42) },
                { OpCode::StoreGlobal(1, 0) },
                { OpCode::UIConst64(0, 99) },
                { OpCode::StoreGlobal(2, 0) },
                { OpCode::Call(1) },
                { OpCode::LoadGlobal(1, 0) },
                { OpCode::StoreGlobal(1, 1)},
                { OpCode::CallNative(0) },
                { OpCode::Return }
            ])
        ]),
        Function::from_basic_blocks(vec! [
            BasicBlock::from_opcodes(vec! [
                { OpCode::LoadGlobal(1, 1) },
                { OpCode::LoadGlobal(2, 2) },
                { OpCode::UIAdd(1, 2) },
                { OpCode::StoreGlobal(0, 0) },
                { OpCode::Return }
            ])
        ])
    ]);
    program.append_native_function(NativeFunction::new("set", &setter));

    let executor = Executor::new();
    executor.eval_program(&ProgramContext::new(&executor, program, None as Option<NoJit>), 0);

    assert_eq!(*result.borrow(), 42 + 99);
}
