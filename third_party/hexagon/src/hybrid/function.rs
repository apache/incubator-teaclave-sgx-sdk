use std::prelude::v1::*;
use super::basic_block::BasicBlock;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Function {
    pub basic_blocks: Vec<BasicBlock>
}

impl Function {
    pub fn from_basic_blocks(basic_blocks: Vec<BasicBlock>) -> Function {
        Function {
            basic_blocks: basic_blocks
        }
    }
}
