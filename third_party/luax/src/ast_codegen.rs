use std::prelude::v1::*;
use std::error::Error;
use std::fmt;
use hexagon::opcode::{OpCode, SelectType};
use ast::{Block, Expr, Stmt, Lhs};
use codegen::{FunctionBuilder, LoopControlInfo};

#[derive(Debug)]
pub struct CodegenError {
    desc: String
}

impl<'a> From<&'a str> for CodegenError {
    fn from(other: &'a str) -> CodegenError {
        CodegenError {
            desc: other.to_string()
        }
    }
}

impl Default for CodegenError {
    fn default() -> Self {
        CodegenError {
            desc: "Error while generating code".into()
        }
    }
}

impl Error for CodegenError {
    fn description(&self) -> &str {
        self.desc.as_str()
    }
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CodegenError: {}", self.desc)
    }
}

impl Lhs {
    fn build_set(&self, fb: &mut FunctionBuilder) -> Result<(), CodegenError> {
        match *self {
            Lhs::Id(ref id) => {
                let loc = fb.get_var_location(id);
                loc.build_set(fb)?;
            },
            Lhs::Index(ref target, ref index) => {
                index.restricted_generate_code(fb)?;
                target.restricted_generate_code(fb)?;
                fb.write_index_set()?;
            }
        }
        Ok(())
    }

    fn build_new_local(&self, fb: &mut FunctionBuilder) -> Result<(), CodegenError> {
        match *self {
            Lhs::Id(ref id) => {
                let loc = fb.create_local(id);
                loc.build_set(fb)?;
            },
            _ => return Err("build_new_local: Unexpected lvalue".into())
        }
        Ok(())
    }
}

pub trait RestrictedGenerateCode {
    fn restricted_generate_code(&self, fb: &mut FunctionBuilder) -> Result<(), CodegenError>;
}

pub trait UnrestrictedGenerateCode {
    fn unrestricted_generate_code(&self, fb: &mut FunctionBuilder) -> Result<(), CodegenError>;
}

impl UnrestrictedGenerateCode for Block {
    fn unrestricted_generate_code(&self, fb: &mut FunctionBuilder) -> Result<(), CodegenError> {
        for stmt in self.statements() {
            stmt.unrestricted_generate_code(fb)?;
        }

        Ok(())
    }
}

impl UnrestrictedGenerateCode for Stmt {
    fn unrestricted_generate_code(&self, fb: &mut FunctionBuilder) -> Result<(), CodegenError> {
        match *self {
            Stmt::Do(ref stmts) => {
                fb.scoped(|fb| -> Result<(), CodegenError> {
                    for stmt in stmts {
                        stmt.unrestricted_generate_code(fb)?;
                    }
                    Ok(())
                })?;
            },
            Stmt::Set(ref lhs, ref exprs) => {
                if lhs.len() != exprs.len() {
                    return Err("Set: lhs & exprs length mismatch".into());
                }
                for i in 0..lhs.len() {
                    exprs[i].restricted_generate_code(fb)?;
                    lhs[i].build_set(fb)?;
                }
            },
            Stmt::While(ref expr, ref blk) => {
                fb.scoped(|fb| -> Result<(), CodegenError> {
                    let expr_check_bb_id = fb.current_basic_block + 1;
                    fb.get_current_bb().opcodes.push(OpCode::Branch(expr_check_bb_id));
                    fb.move_forward();

                    expr.restricted_generate_code(fb)?;

                    let break_point_bb_id = fb.current_basic_block + 1;
                    fb.move_forward();

                    let body_begin_bb_id = fb.current_basic_block + 1;
                    fb.move_forward();

                    fb.with_lci(LoopControlInfo {
                        break_point: break_point_bb_id,
                        continue_point: expr_check_bb_id
                    }, |fb| blk.unrestricted_generate_code(fb))?;

                    fb.get_current_bb().opcodes.push(OpCode::Branch(expr_check_bb_id));

                    let end_bb_id = fb.current_basic_block + 1;
                    fb.move_forward();

                    fb.basic_blocks[break_point_bb_id].opcodes.push(OpCode::Branch(end_bb_id));
                    fb.basic_blocks[expr_check_bb_id].opcodes.push(OpCode::ConditionalBranch(
                        body_begin_bb_id,
                        end_bb_id
                    ));
                    Ok(())
                })?;
            },
            Stmt::Repeat(ref blk, ref expr) => {
                fb.scoped(|fb| -> Result<(), CodegenError> {
                    let before_bb_id = fb.current_basic_block;

                    let continue_point_bb_id = fb.current_basic_block + 1;
                    fb.move_forward();

                    let break_point_bb_id = fb.current_basic_block + 1;
                    fb.move_forward();

                    let body_begin_bb_id = fb.current_basic_block + 1;
                    fb.move_forward();

                    fb.basic_blocks[before_bb_id].opcodes.push(OpCode::Branch(body_begin_bb_id));

                    fb.with_lci(LoopControlInfo {
                        break_point: break_point_bb_id,
                        continue_point: continue_point_bb_id
                    }, |fb| blk.unrestricted_generate_code(fb))?;

                    let body_end_bb_id = fb.current_basic_block;

                    let expr_check_bb_id = fb.current_basic_block + 1;
                    fb.move_forward();

                    expr.restricted_generate_code(fb)?;

                    let end_bb_id = fb.current_basic_block + 1;
                    fb.move_forward();

                    fb.basic_blocks[continue_point_bb_id].opcodes.push(OpCode::Branch(expr_check_bb_id));
                    fb.basic_blocks[break_point_bb_id].opcodes.push(OpCode::Branch(end_bb_id));
                    fb.basic_blocks[body_end_bb_id].opcodes.push(OpCode::Branch(expr_check_bb_id));
                    fb.basic_blocks[expr_check_bb_id].opcodes.push(OpCode::ConditionalBranch(
                        end_bb_id,
                        body_begin_bb_id
                    ));
                    Ok(())
                })?;
            },
            Stmt::If(ref branches, ref else_branch) => {
                let before_bb_id = fb.current_basic_block;

                fb.move_forward();
                let terminator_bb_id = fb.current_basic_block;

                let mut branch_begin_bbs: Vec<usize> = Vec::new();
                for &(ref expr, _) in branches {
                    fb.move_forward();
                    expr.restricted_generate_code(fb)?;
                    branch_begin_bbs.push(fb.current_basic_block);
                }

                if let Some(ref else_blk) = *else_branch {
                    fb.move_forward();
                    let else_begin = fb.current_basic_block;
                    fb.scoped(|fb| else_blk.unrestricted_generate_code(fb))?;
                    fb.get_current_bb().opcodes.push(OpCode::Branch(terminator_bb_id));
                    branch_begin_bbs.push(else_begin);
                } else {
                    branch_begin_bbs.push(terminator_bb_id);
                }

                assert!(branch_begin_bbs.len() == branches.len() + 1);
                fb.basic_blocks[before_bb_id].opcodes.push(OpCode::Branch(branch_begin_bbs[0]));

                for i in 0..branches.len() {
                    let &(_, ref blk) = &branches[i];
                    fb.move_forward();
                    let current_bb = fb.current_basic_block;
                    let checker_bb = branch_begin_bbs[i];
                    fb.basic_blocks[checker_bb].opcodes.push(OpCode::ConditionalBranch(
                        current_bb,
                        branch_begin_bbs[i + 1]
                    ));
                    fb.scoped(|fb| blk.unrestricted_generate_code(fb))?;
                    fb.get_current_bb().opcodes.push(OpCode::Branch(terminator_bb_id));
                }

                fb.move_forward();
                let end_bb_id = fb.current_basic_block;
                fb.basic_blocks[terminator_bb_id].opcodes.push(OpCode::Branch(end_bb_id));
            },
            Stmt::Local(ref lhs, ref exprs) => {
                if lhs.len() != exprs.len() {
                    return Err("Local: lhs & exprs length mismatch".into());
                }
                for i in 0..lhs.len() {
                    exprs[i].restricted_generate_code(fb)?;
                    lhs[i].build_new_local(fb)?;
                }
            },
            Stmt::Call(ref target, ref args) => {
                Expr::Call(Box::new(target.clone()), args.clone()).restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Pop);
            },
            Stmt::Return(ref v) => {
                if v.len() == 0 {
                    fb.get_current_bb().opcodes.push(OpCode::LoadNull);
                    fb.get_current_bb().opcodes.push(OpCode::Return);
                    fb.move_forward();
                } else if v.len() == 1 {
                    v[0].restricted_generate_code(fb)?;
                    fb.get_current_bb().opcodes.push(OpCode::Return);
                    fb.move_forward();
                } else {
                    return Err("Multiple return values is not supported for now".into());
                }
            },
            Stmt::Break => {
                fb.write_break()?;
            },
            _ => return Err("Not implemented".into())
        }

        Ok(())
    }
}

impl RestrictedGenerateCode for Expr {
    fn restricted_generate_code(&self, fb: &mut FunctionBuilder) -> Result<(), CodegenError> {
        match *self {
            Expr::Nil => fb.get_current_bb().opcodes.push(OpCode::LoadNull),
            Expr::Boolean(v) => fb.get_current_bb().opcodes.push(OpCode::LoadBool(v)),
            Expr::Number(v) => fb.get_current_bb().opcodes.push(OpCode::LoadFloat(v)),
            Expr::String(ref s) => fb.get_current_bb().opcodes.push(OpCode::LoadString(s.clone())),
            Expr::Function(ref vlhs, ref blk) => {
                let new_builder = fb.get_module_builder().new_function();

                let mut arg_names: Vec<String> = Vec::new();
                for lhs in vlhs {
                    if let Some(id) = lhs.id() {
                        arg_names.push(id.to_string());
                    } else {
                        return Err("Expecting id in function signature".into());
                    }
                }

                let fn_id = new_builder.build(blk, arg_names)?;

                fb.write_function_load(fn_id)?;
            },
            Expr::Table(ref elems) => {
                fb.write_array_create()?;
                for (_i, v) in elems.iter().enumerate() {
                    fb.get_current_bb().opcodes.push(OpCode::Dup);
                    v.restricted_generate_code(fb)?;
                    fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                    fb.write_array_push()?;
                }
                fb.write_table_create()?;
                fb.get_current_bb().opcodes.extend(vec! [
                    OpCode::Dup,
                    OpCode::Rotate3,
                    OpCode::Rotate2,
                    OpCode::LoadString("__copy_from_array__".into()),
                    OpCode::LoadNull,
                    OpCode::Rotate3,
                    OpCode::CallField(1),
                    OpCode::Pop
                ]);
            },
            Expr::Add(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.get_current_bb().opcodes.push(OpCode::Add);
            },
            Expr::Sub(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.get_current_bb().opcodes.push(OpCode::Sub);
            },
            Expr::Mul(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.get_current_bb().opcodes.push(OpCode::Mul);
            },
            Expr::Div(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.get_current_bb().opcodes.push(OpCode::Div);
            },
            Expr::Idiv(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.get_current_bb().opcodes.push(OpCode::IntDiv);
            },
            Expr::Mod(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.get_current_bb().opcodes.push(OpCode::Mod);
            },
            Expr::Pow(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.get_current_bb().opcodes.push(OpCode::Pow);
            },
            Expr::Concat(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.write_concat()?;
            },
            Expr::Eq(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.get_current_bb().opcodes.push(OpCode::TestEq);
            },
            Expr::Ne(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.get_current_bb().opcodes.push(OpCode::TestNe);
            },
            Expr::Lt(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.get_current_bb().opcodes.push(OpCode::TestLt);
            },
            Expr::Gt(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.get_current_bb().opcodes.push(OpCode::TestGt);
            },
            Expr::Le(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.get_current_bb().opcodes.push(OpCode::TestLe);
            },
            Expr::Ge(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.get_current_bb().opcodes.push(OpCode::TestGe);
            },
            Expr::Not(ref v) => {
                v.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Not);
            },
            Expr::Unm(ref v) => {
                v.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::LoadFloat(0.0));
                fb.get_current_bb().opcodes.push(OpCode::Sub);
            },
            Expr::And(ref left, ref right) => {
                let begin = fb.get_current_bb().opcodes.len();
                left.restricted_generate_code(fb)?;
                let left_opcodes = fb.get_current_bb().detach_opcodes(begin);
                assert!(begin == fb.get_current_bb().opcodes.len());

                right.restricted_generate_code(fb)?;
                let right_opcodes = fb.get_current_bb().detach_opcodes(begin);
                assert!(begin == fb.get_current_bb().opcodes.len());

                fb.get_current_bb().opcodes.push(OpCode::Select(
                    SelectType::And,
                    left_opcodes,
                    right_opcodes
                ));
            },
            Expr::Or(ref left, ref right) => {
                let begin = fb.get_current_bb().opcodes.len();
                left.restricted_generate_code(fb)?;
                let left_opcodes = fb.get_current_bb().detach_opcodes(begin);
                assert!(begin == fb.get_current_bb().opcodes.len());

                right.restricted_generate_code(fb)?;
                let right_opcodes = fb.get_current_bb().detach_opcodes(begin);
                assert!(begin == fb.get_current_bb().opcodes.len());

                fb.get_current_bb().opcodes.push(OpCode::Select(
                    SelectType::Or,
                    left_opcodes,
                    right_opcodes
                ));
            },
            Expr::Call(ref target, ref args) => {
                for arg in args {
                    arg.restricted_generate_code(fb)?;
                }
                if args.len() > 0 {
                    fb.get_current_bb().opcodes.push(OpCode::RotateReverse(args.len()));
                }
                fb.get_current_bb().opcodes.push(OpCode::LoadNull);
                target.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Call(args.len()));
            },
            Expr::Pair(ref left, ref right) => {
                left.restricted_generate_code(fb)?;
                right.restricted_generate_code(fb)?;
                fb.get_current_bb().opcodes.push(OpCode::Rotate2);
                fb.write_pair_create()?;
            },
            Expr::Id(ref k) => {
                let loc = fb.get_var_location(k.as_str());
                loc.build_get(fb)?;
            },
            Expr::Index(ref target, ref index) => {
                index.restricted_generate_code(fb)?;
                target.restricted_generate_code(fb)?;
                fb.write_index_get()?;
            },
            Expr::Dots => {
                return Err("Dots: Not implemented".into());
            }
        }

        Ok(())
    }
}
