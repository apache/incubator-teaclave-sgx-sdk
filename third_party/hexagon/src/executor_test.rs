use executor::Executor;
use opcode::OpCode;
use basic_block::BasicBlock;
use function::Function;
use value::ValueContext;

#[test]
fn test_executor() {
    let executor = Executor::new();
    let mut handle = executor.handle_mut();

    const END: i64 = 100000;

    let mut sum_fn = Box::new(Function::from_basic_blocks(vec! [
        // bb 0
        BasicBlock::from_opcodes(vec! [
            { OpCode::InitLocal(8) },
            { OpCode::LoadInt(0) }, // current (initial) value (exclusive)
            { OpCode::SetLocal(0) },
            { OpCode::LoadInt(END) }, // end value (inclusive)
            { OpCode::SetLocal(1) },
            { OpCode::LoadInt(0) }, // sum
            { OpCode::SetLocal(2) },
            { OpCode::Branch(1) }
        ]),
        // bb 1
        BasicBlock::from_opcodes(vec! [
            { OpCode::GetLocal(1) },
            { OpCode::GetLocal(0) },
            { OpCode::TestLt },
            { OpCode::Not },
            { OpCode::ConditionalBranch(3, 2) }
        ]),
        // bb 2
        BasicBlock::from_opcodes(vec! [
            { OpCode::LoadInt(1) },
            { OpCode::GetLocal(0) },
            { OpCode::IntAdd },
            { OpCode::Dup },
            { OpCode::SetLocal(0) },
            { OpCode::GetLocal(2) },
            { OpCode::IntAdd },
            { OpCode::SetLocal(2) },
            { OpCode::Branch(1) }
        ]),
        // bb 3
        BasicBlock::from_opcodes(vec! [
            { OpCode::GetLocal(2) },
            { OpCode::Return }
        ])
    ]));
    sum_fn.enable_optimization();
    handle.create_static_object("sum", sum_fn);

    let blocks: Vec<BasicBlock> = vec! [
        BasicBlock::from_opcodes(vec! [
            { OpCode::InitLocal(8) },
            { OpCode::LoadString("sum".to_string()) },
            { OpCode::GetStatic },
            { OpCode::SetLocal(0) },
            { OpCode::LoadNull },
            { OpCode::GetLocal(0) },
            { OpCode::Call(0) },
            { OpCode::LoadString("output".to_string()) },
            { OpCode::SetStatic },
            { OpCode::LoadNull },
            { OpCode::Return }
        ])
    ];
    handle.create_static_object("entry", Box::new(Function::from_basic_blocks(blocks)));
    match handle.run_callable("entry") {
        Ok(_) => {},
        Err(e) => panic!(e.unwrap().to_string())
    }

    handle.gc();

    let result_value = handle.get_static_object("output").unwrap();
    let result = ValueContext::new(
        &result_value,
        handle.get_object_pool()
    ).to_i64();

    assert_eq!(result, (1 + END) * END / 2);
}
