use std::prelude::v1::*;
use std::io;
use super::{Deserialize, Serialize, Error, VarUint32, CountedList, InitExpr, CountedListWriter};

/// Entry in the element section.
#[derive(Debug, Clone)]
pub struct ElementSegment {
	index: u32,
	offset: InitExpr,
	members: Vec<u32>,
}

impl ElementSegment {
	/// New element segment.
	pub fn new(index: u32, offset: InitExpr, members: Vec<u32>) -> Self {
		ElementSegment { index: index, offset: offset, members: members }
	}

	/// Sequence of function indices.
	pub fn members(&self) -> &[u32] { &self.members }

	/// Sequence of function indices (mutable)
	pub fn members_mut(&mut self) -> &mut Vec<u32> { &mut self.members }

	/// Table index (currently valid only value of `0`)
	pub fn index(&self) -> u32 { self.index }

	/// An i32 initializer expression that computes the offset at which to place the elements.
	pub fn offset(&self) -> &InitExpr { &self.offset }

	/// An i32 initializer expression that computes the offset at which to place the elements (mutable)
	pub fn offset_mut(&mut self) -> &mut InitExpr { &mut self.offset }
}

impl Deserialize for ElementSegment {
	 type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let index = VarUint32::deserialize(reader)?;
		let offset = InitExpr::deserialize(reader)?;
		let funcs: Vec<u32> = CountedList::<VarUint32>::deserialize(reader)?
			.into_inner()
			.into_iter()
			.map(Into::into)
			.collect();

		Ok(ElementSegment {
			index: index.into(),
			offset: offset,
			members: funcs,
		})
	}
}

impl Serialize for ElementSegment {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		VarUint32::from(self.index).serialize(writer)?;
		self.offset.serialize(writer)?;
		let data = self.members;
		let counted_list = CountedListWriter::<VarUint32, _>(
			data.len(),
			data.into_iter().map(Into::into),
		);
		counted_list.serialize(writer)?;
		Ok(())
	}
}

/// Data segment definition.
#[derive(Clone, Debug)]
pub struct DataSegment {
	index: u32,
	offset: InitExpr,
	value: Vec<u8>,
}

impl DataSegment {
	/// New data segments.
	pub fn new(index: u32, offset: InitExpr, value: Vec<u8>) -> Self {
		DataSegment {
			index: index,
			offset: offset,
			value: value,
		}
	}

	/// Linear memory index (currently the only valid value is `0`).
	pub fn index(&self) -> u32 { self.index }

	/// An i32 initializer expression that computes the offset at which to place the data.
	pub fn offset(&self) -> &InitExpr { &self.offset }

	/// An i32 initializer expression that computes the offset at which to place the data (mutable)
	pub fn offset_mut(&mut self) -> &mut InitExpr { &mut self.offset }

	/// Initial value of the data segment.
	pub fn value(&self) -> &[u8] { &self.value }

	/// Initial value of the data segment (mutable).
	pub fn value_mut(&mut self) -> &mut Vec<u8> { &mut self.value }
}

impl Deserialize for DataSegment {
	 type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let index = VarUint32::deserialize(reader)?;
		let offset = InitExpr::deserialize(reader)?;
		let value_len = u32::from(VarUint32::deserialize(reader)?) as usize;
		let value_buf = buffered_read!(65536, value_len, reader);

		Ok(DataSegment {
			index: index.into(),
			offset: offset,
			value: value_buf,
		})
	}
}

impl Serialize for DataSegment {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		VarUint32::from(self.index).serialize(writer)?;
		self.offset.serialize(writer)?;

		let value = self.value;
		VarUint32::from(value.len()).serialize(writer)?;
		writer.write_all(&value[..])?;
		Ok(())
	}
}
