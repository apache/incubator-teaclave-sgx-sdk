use super::executor::Executor;
use super::program::Program;
use super::jit::JitProvider;

pub trait CommonProgramContext {
    fn get_executor(&self) -> &Executor;
    fn get_program(&self) -> &Program;
    fn get_jit_provider(&self) -> Option<&JitProvider>;
}

pub struct ProgramContext<'a, TJitProvider: JitProvider> {
    pub executor: &'a Executor,
    pub program: Program<'a>,
    jit_provider: Option<TJitProvider>
}

impl<'a, TJitProvider: JitProvider> ProgramContext<'a, TJitProvider> {
    pub fn new(
        executor: &'a Executor,
        program: Program<'a>,
        jit_provider: Option<TJitProvider>
    ) -> ProgramContext<'a, TJitProvider> {
        ProgramContext {
            executor: executor,
            program: program,
            jit_provider: jit_provider
        }
    }
}

impl<'a, TJitProvider: JitProvider> CommonProgramContext for ProgramContext<'a, TJitProvider> {
    fn get_executor(&self) -> &Executor {
        self.executor
    }

    fn get_program(&self) -> &Program {
        &self.program
    }

    fn get_jit_provider(&self) -> Option<&JitProvider> {
        self.jit_provider.as_ref().map(|v| v as &JitProvider)
    }
}
