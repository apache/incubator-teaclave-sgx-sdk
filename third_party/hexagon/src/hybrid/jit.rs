use super::program_context::CommonProgramContext;

pub trait JitProvider {
    /// Invokes a function.
    ///
    /// If the JIT provider is ready for the invoke request, execute the function
    /// and return true. Otherwise, return false and return immediately.
    ///
    /// The JIT can schedule compilation based on record of calls to this method.
    fn invoke_function(&self, _ctx: &CommonProgramContext, _id: usize) -> bool;
}

pub struct NoJit {
    _no_construct: ()
}

impl JitProvider for NoJit {
    fn invoke_function(&self, _ctx: &CommonProgramContext, _id: usize) -> bool {
        unreachable!()
    }
}
