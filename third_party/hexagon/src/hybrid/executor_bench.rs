use test::Bencher;
use super::executor::Executor;
use super::program::Program;
use super::basic_block::BasicBlock;
use super::opcode::OpCode;
use super::function::Function;
use super::program_context::ProgramContext;
use super::jit::NoJit;

#[bench]
fn bench_invoke(b: &mut Bencher) {
    let executor = Executor::new();
    let program = Program::from_functions(vec! [
        Function::from_basic_blocks(vec! [
            BasicBlock::from_opcodes(vec! [
                { OpCode::Return }
            ])
        ])
    ]);
    let ctx = ProgramContext::new(&executor, program, None as Option<NoJit>);
    b.iter(|| {
        executor.eval_program(&ctx, 0)
    });
}
