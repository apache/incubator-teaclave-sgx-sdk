use std::prelude::v1::*;
use super::invoke::{Identity, Invoke};
use elements;

/// Data segment builder
pub struct DataSegmentBuilder<F=Identity> {
	callback: F,
	// todo: add mapper once multiple memory refs possible
	mem_index: u32,
	offset: elements::InitExpr,
	value: Vec<u8>,
}

impl DataSegmentBuilder {
	/// New data segment builder
	pub fn new() -> Self {
		DataSegmentBuilder::with_callback(Identity)
	}
}

impl<F> DataSegmentBuilder<F> {
	/// New data segment builder inside the chain context
	pub fn with_callback(callback: F) -> Self {
		DataSegmentBuilder {
			callback: callback,
			mem_index: 0,
			offset: elements::InitExpr::empty(),
			value: Vec::new(),
		}
	}

	/// Set offset initialization opcode. `End` opcode will be added automatically.
	pub fn offset(mut self, opcode: elements::Opcode) -> Self {
		self.offset = elements::InitExpr::new(vec![opcode, elements::Opcode::End]);
		self
	}

	/// Set the bytes value of the segment
	pub fn value(mut self, value: Vec<u8>) -> Self {
		self.value = value;
		self
	}
}

impl<F> DataSegmentBuilder<F> where F: Invoke<elements::DataSegment> {
	/// Finish current builder, spawning resulting struct
	pub fn build(self) -> F::Result {
		self.callback.invoke(
			elements::DataSegment::new(
				self.mem_index,
				self.offset,
				self.value,
			)
		)
	}
}
