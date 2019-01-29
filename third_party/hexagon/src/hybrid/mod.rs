pub mod basic_block;
pub mod executor;
pub mod function;
pub mod jit;
pub mod opcode;
pub mod page_table;
pub mod program_context;
pub mod program;
pub mod type_cast;

#[cfg(test)]
mod executor_bench;

#[cfg(test)]
mod executor_test;

#[cfg(test)]
mod jit_test;

#[cfg(test)]
mod page_table_bench;

#[cfg(test)]
mod page_table_test;

#[cfg(test)]
mod program_test;
