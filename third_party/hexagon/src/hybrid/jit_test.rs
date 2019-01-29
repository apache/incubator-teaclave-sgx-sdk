use std::panic::{catch_unwind, AssertUnwindSafe};
use super::jit::JitProvider;
use super::program::Program;
use super::program_context::{ProgramContext, CommonProgramContext};
use super::executor::Executor;
use super::function::Function;
use super::basic_block::BasicBlock;
use super::opcode::OpCode;

struct TestJitProvider {

}

impl JitProvider for TestJitProvider {
    fn invoke_function(&self, ctx: &CommonProgramContext, id: usize) -> bool {
        panic!("OK");
    }
}

#[test]
fn test_jit() {
    let program = Program::from_functions(vec! [
        Function::from_basic_blocks(vec! [
            BasicBlock::from_opcodes(vec! [
                { OpCode::Return }
            ])
        ])
    ]);
    let executor = Executor::new();
    let ctx = ProgramContext::new(&executor, program, Some(TestJitProvider {}));

    match catch_unwind(AssertUnwindSafe(|| executor.eval_program(&ctx, 0))) {
        Ok(_) => panic!("Unwind expected"),
        Err(e) => {
            let e = e.downcast::<&'static str>().unwrap();
            assert_eq!(*e, "OK");
        }
    }
}
