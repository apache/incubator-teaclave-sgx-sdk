use test::Bencher;
use executor::Executor;
use opcode::OpCode;
use basic_block::BasicBlock;
use function::Function;
use value::{Value, ValueContext};

#[bench]
fn run(b: &mut Bencher) {
    let executor = Executor::new();
    let mut handle = executor.handle_mut();

    let mut entry_fn = Function::from_basic_blocks(vec! [
        BasicBlock::from_opcodes(vec! [
            { OpCode::GetArgument(0) },
            { OpCode::GetArgument(1) },
            { OpCode::GetArgument(2) }, // (0, 1, 2)
            { OpCode::Dup }, // (0, 1, 2, 2)
            { OpCode::Rotate3 }, // (0, 2, 2, 1)
            { OpCode::Rotate2 }, // (0, 2, 1, 2)
            { OpCode::RotateReverse(4) }, // (2, 1, 2, 0)
            { OpCode::IntAdd }, // (2, 1, 0 + 2)
            { OpCode::IntMul }, // (2, (0 + 2) * 1)
            { OpCode::IntSub }, // ((0 + 2) * 1 - 2)
            { OpCode::Return }
        ])
    ]);

    entry_fn.enable_optimization();

    let entry_obj_id = handle.get_object_pool_mut().allocate(Box::new(entry_fn));
    println!("{:?}",
        handle.get_object_pool().get_direct_typed::<Function>(entry_obj_id).unwrap()
            .to_virtual_info().unwrap()
    );
    

    let entry = Value::Object(entry_obj_id);
    let mut ret = Value::Null;
    b.iter(|| {
        handle.invoke(entry, Value::Null, None, &[
            Value::Int(2),
            Value::Int(6),
            Value::Int(5)
        ]);
        ret = handle.get_current_frame().pop_exec();
    });

    assert!(ret == Value::Int(37));
}
