use std::prelude::v1::*;
use super::opcode::OpCode;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BasicBlock {
    pub opcodes: Vec<OpCode>
}

impl BasicBlock {
    pub fn from_opcodes(opcodes: Vec<OpCode>) -> BasicBlock {
        BasicBlock {
            opcodes: opcodes
        }
    }
}
