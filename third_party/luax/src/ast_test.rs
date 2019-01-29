use std::prelude::v1::*;
use ast::*;
use serde_json;
use test_programs;

#[test]
fn test_ast_build() {
    let mut ast: Block = serde_json::from_str(test_programs::get("simple_local")).unwrap();
    println!("{:?}", ast);

    ast = serde_json::from_str(test_programs::get("function_def_call")).unwrap();
    println!("{:?}", ast);

    ast = serde_json::from_str(test_programs::get("loops")).unwrap();
    println!("{:?}", ast);

    ast = serde_json::from_str(test_programs::get("arithmetic")).unwrap();
    println!("{:?}", ast);
}
