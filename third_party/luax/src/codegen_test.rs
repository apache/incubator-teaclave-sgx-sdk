use std::prelude::v1::*;
use std::panic::{catch_unwind, AssertUnwindSafe, resume_unwind};
use ast;
use codegen;
use ast_codegen;
use runtime;
use serde_json;
use test_programs;
use hexagon::executor::{Executor, ExecutorImpl};
use hexagon::errors::VMError;

fn gen_and_run(ast: ast::Block) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let module = codegen::ModuleBuilder::new();
        let fn_builder = codegen::FunctionBuilder::new(&module);
        let fn_id = fn_builder.build(&ast, Vec::new()).unwrap();

        let mut executor = ExecutorImpl::new();
        runtime::invoke(&mut executor, module, fn_id);
    }));
    if let Err(e) = result {
        match e.downcast::<VMError>() {
            Ok(e) => panic!("VMError: {}", e.unwrap().to_string()),
            Err(v) => resume_unwind(v)
        }
    }
}

#[test]
fn run_simple_local() {
    gen_and_run(serde_json::from_str(test_programs::get("simple_local")).unwrap());
}

#[test]
fn run_function_def_call() {
    gen_and_run(serde_json::from_str(test_programs::get("function_def_call")).unwrap());
}

/*
#[test]
fn run_loops() {
    gen_and_run(serde_json::from_str(test_programs::get("loops")).unwrap());
}
*/

#[test]
fn run_arithmetic() {
    gen_and_run(serde_json::from_str(test_programs::get("arithmetic")).unwrap());
}
