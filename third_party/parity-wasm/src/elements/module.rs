use std::prelude::v1::*;
use std::io;
use byteorder::{LittleEndian, ByteOrder};

use super::{Deserialize, Serialize, Error, Uint32, External};
use super::section::{
	Section, CodeSection, TypeSection, ImportSection, ExportSection, FunctionSection,
	GlobalSection, TableSection, ElementSection, DataSection, MemorySection
};
use super::name_section::NameSection;
use super::reloc_section::RelocSection;

const WASM_MAGIC_NUMBER: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];

/// WebAssembly module
#[derive(Debug, Clone)]
pub struct Module {
	magic: u32,
	version: u32,
	sections: Vec<Section>,
}

#[derive(Debug, Clone, Copy)]
/// Type of the import entry to count
pub enum ImportCountType {
	/// Count functions
	Function,
	/// Count globals
	Global,
	/// Count tables
	Table,
	/// Count memories
	Memory,
}

impl Default for Module {
	fn default() -> Self {
		Module {
			magic: LittleEndian::read_u32(&WASM_MAGIC_NUMBER),
			version: 1,
			sections: Vec::with_capacity(16),
		}
	}
}

impl Module {
	/// New module with sections
	pub fn new(sections: Vec<Section>) -> Self {
		Module {
			sections: sections, ..Default::default()
		}
	}

	/// Destructure the module, yielding sections
	pub fn into_sections(self) -> Vec<Section> {
		self.sections
	}

	/// Version of module.
	pub fn version(&self) -> u32 { self.version }

	/// Sections list.
	/// Each known section is optional and may appear at most once.
	pub fn sections(&self) -> &[Section] {
		&self.sections
	}

	/// Sections list (mutable)
	/// Each known section is optional and may appear at most once.
	pub fn sections_mut(&mut self) -> &mut Vec<Section> {
		&mut self.sections
	}

	/// Code section reference, if any.
	pub fn code_section(&self) -> Option<&CodeSection> {
		for section in self.sections() {
			if let &Section::Code(ref code_section) = section { return Some(code_section); }
		}
		None
	}

	/// Code section mutable reference, if any.
	pub fn code_section_mut(&mut self) -> Option<&mut CodeSection> {
		for section in self.sections_mut() {
			if let Section::Code(ref mut code_section) = *section { return Some(code_section); }
		}
		None
	}

	/// Types section reference, if any.
	pub fn type_section(&self) -> Option<&TypeSection> {
		for section in self.sections() {
			if let &Section::Type(ref type_section) = section { return Some(type_section); }
		}
		None
	}

	/// Types section mutable reference, if any.
	pub fn type_section_mut(&mut self) -> Option<&mut TypeSection> {
		for section in self.sections_mut() {
			if let Section::Type(ref mut type_section) = *section { return Some(type_section); }
		}
		None
	}

	/// Imports section reference, if any.
	pub fn import_section(&self) -> Option<&ImportSection> {
		for section in self.sections() {
			if let &Section::Import(ref import_section) = section { return Some(import_section); }
		}
		None
	}

	/// Imports section mutable reference, if any.
	pub fn import_section_mut(&mut self) -> Option<&mut ImportSection> {
		for section in self.sections_mut() {
			if let Section::Import(ref mut import_section) = *section { return Some(import_section); }
		}
		None
	}

	/// Globals section reference, if any.
	pub fn global_section(&self) -> Option<&GlobalSection> {
		for section in self.sections() {
			if let &Section::Global(ref section) = section { return Some(section); }
		}
		None
	}

		/// Globals section mutable reference, if any.
	pub fn global_section_mut(&mut self) -> Option<&mut GlobalSection> {
		for section in self.sections_mut() {
			if let Section::Global(ref mut section) = *section { return Some(section); }
		}
		None
	}


	/// Exports section reference, if any.
	pub fn export_section(&self) -> Option<&ExportSection> {
		for section in self.sections() {
			if let &Section::Export(ref export_section) = section { return Some(export_section); }
		}
		None
	}

	/// Exports section mutable reference, if any.
	pub fn export_section_mut(&mut self) -> Option<&mut ExportSection> {
		for section in self.sections_mut() {
			if let Section::Export(ref mut export_section) = *section { return Some(export_section); }
		}
		None
	}

	/// Table section reference, if any.
	pub fn table_section(&self) -> Option<&TableSection> {
		for section in self.sections() {
			if let &Section::Table(ref section) = section { return Some(section); }
		}
		None
	}

	/// Table section mutable reference, if any.
	pub fn table_section_mut(&mut self) -> Option<&mut TableSection> {
		for section in self.sections_mut() {
			if let Section::Table(ref mut section) = *section { return Some(section); }
		}
		None
	}

	/// Data section reference, if any.
	pub fn data_section(&self) -> Option<&DataSection> {
		for section in self.sections() {
			if let &Section::Data(ref section) = section { return Some(section); }
		}
		None
	}

	/// Data section mutable reference, if any.
	pub fn data_section_mut(&mut self) -> Option<&mut DataSection> {
		for section in self.sections_mut() {
			if let Section::Data(ref mut section) = *section { return Some(section); }
		}
		None
	}

	/// Element section reference, if any.
	pub fn elements_section(&self) -> Option<&ElementSection> {
		for section in self.sections() {
			if let &Section::Element(ref section) = section { return Some(section); }
		}
		None
	}

	/// Element section mutable reference, if any.
	pub fn elements_section_mut(&mut self) -> Option<&mut ElementSection> {
		for section in self.sections_mut() {
			if let Section::Element(ref mut section) = *section { return Some(section); }
		}
		None
	}

	/// Memory section reference, if any.
	pub fn memory_section(&self) -> Option<&MemorySection> {
		for section in self.sections() {
			if let &Section::Memory(ref section) = section { return Some(section); }
		}
		None
	}

	/// Memory section mutable reference, if any.
	pub fn memory_section_mut(&mut self) -> Option<&mut MemorySection> {
		for section in self.sections_mut() {
			if let Section::Memory(ref mut section) = *section { return Some(section); }
		}
		None
	}

	/// Functions signatures section reference, if any.
	pub fn function_section(&self) -> Option<&FunctionSection> {
		for section in self.sections() {
			if let &Section::Function(ref sect) = section { return Some(sect); }
		}
		None
	}

	/// Functions signatures section mutable reference, if any.
	pub fn function_section_mut(&mut self) -> Option<&mut FunctionSection> {
		for section in self.sections_mut() {
			if let Section::Function(ref mut sect) = *section { return Some(sect); }
		}
		None
	}

	/// Start section, if any.
	pub fn start_section(&self) -> Option<u32> {
		for section in self.sections() {
			if let &Section::Start(sect) = section { return Some(sect); }
		}
		None
	}

	/// Functions signatures section reference, if any.
	/// NOTE: name section is not parsed by default so `names_section` could return None even if name section exists.
	/// Call `parse_names` to parse name section
	pub fn names_section(&self) -> Option<&NameSection> {
		for section in self.sections() {
			if let Section::Name(ref sect) = *section { return Some(sect); }
		}
		None
	}

	/// Functions signatures section mutable reference, if any.
	/// NOTE: name section is not parsed by default so `names_section` could return None even if name section exists.
	/// Call `parse_names` to parse name section
	pub fn names_section_mut(&mut self) -> Option<&mut NameSection> {
		for section in self.sections_mut() {
			if let Section::Name(ref mut sect) = *section { return Some(sect); }
		}
		None
	}

	/// Try to parse name section in place
	/// Corresponding custom section with proper header will convert to name sections
	/// If some of them will fail to be decoded, Err variant is returned with the list of
	/// (index, Error) tuples of failed sections.
	pub fn parse_names(mut self) -> Result<Self, (Vec<(usize, Error)>, Self)> {
		let mut parse_errors = Vec::new();

		for i in 0..self.sections.len() {
			if let Some(name_section) = {
				let section = self.sections.get(i).expect("cannot fail because i in range 0..len; qed");
				if let Section::Custom(ref custom) = *section {
					if custom.name() == "name" {
						let mut rdr = io::Cursor::new(custom.payload());
						let name_section = match NameSection::deserialize(&self, &mut rdr) {
							Ok(ns) => ns,
							Err(e) => { parse_errors.push((i, e)); continue; }
						};
						Some(name_section)
					} else {
						None
					}
				} else { None }
			} {
				*self.sections.get_mut(i).expect("cannot fail because i in range 0..len; qed") = Section::Name(name_section);
			}
		}

		if parse_errors.len() > 0 {
			Err((parse_errors, self))
		} else {
			Ok(self)
		}
	}

	/// Try to parse reloc section in place
	/// Corresponding custom section with proper header will convert to reloc sections
	/// If some of them will fail to be decoded, Err variant is returned with the list of
	/// (index, Error) tuples of failed sections.
	pub fn parse_reloc(mut self) -> Result<Self, (Vec<(usize, Error)>, Self)> {
		let mut parse_errors = Vec::new();

		for (i, section) in self.sections.iter_mut().enumerate() {
			if let Some(relocation_section) = {
				if let Section::Custom(ref custom) = *section {
					if custom.name().starts_with("reloc.") {
						let mut rdr = io::Cursor::new(custom.payload());
						let reloc_section = match RelocSection::deserialize(custom.name().to_owned(), &mut rdr) {
							Ok(reloc_section) => reloc_section,
							Err(e) => { parse_errors.push((i, e)); continue; }
						};
						if rdr.position() != custom.payload().len() as u64 {
							parse_errors.push((i, io::Error::from(io::ErrorKind::InvalidData).into()));
							continue;
						}
						Some(Section::Reloc(reloc_section))
					}
					else {
						None
					}
				}
				else {
					None
				}
			} {
				*section = relocation_section;
			}
		}

		if parse_errors.len() > 0 {
			Err((parse_errors, self))
		} else {
			Ok(self)
		}
	}

	/// Count imports by provided type
	pub fn import_count(&self, count_type: ImportCountType) -> usize {
		self.import_section()
			.map(|is|
				is.entries().iter().filter(|import| match (count_type, *import.external()) {
					(ImportCountType::Function, External::Function(_)) => true,
					(ImportCountType::Global, External::Global(_)) => true,
					(ImportCountType::Table, External::Table(_)) => true,
					(ImportCountType::Memory, External::Memory(_)) => true,
					_ => false
				}).count())
			.unwrap_or(0)
	}

	/// Query functions space
	pub fn functions_space(&self) -> usize {
		self.import_count(ImportCountType::Function) +
			self.function_section().map(|fs| fs.entries().len()).unwrap_or(0)
	}

	/// Query globals space
	pub fn globals_space(&self) -> usize {
		self.import_count(ImportCountType::Global) +
			self.global_section().map(|gs| gs.entries().len()).unwrap_or(0)
	}

	/// Query table space
	pub fn table_space(&self) -> usize {
		self.import_count(ImportCountType::Table) +
			self.table_section().map(|ts| ts.entries().len()).unwrap_or(0)
	}

	/// Query memory space
	pub fn memory_space(&self) -> usize {
		self.import_count(ImportCountType::Memory) +
			self.memory_section().map(|ms| ms.entries().len()).unwrap_or(0)
	}
}

impl Deserialize for Module {
	type Error = super::Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let mut sections = Vec::new();

		let mut magic = [0u8; 4];
		reader.read(&mut magic)?;
		if magic != WASM_MAGIC_NUMBER {
			return Err(Error::InvalidMagic);
		}

		let version: u32 = Uint32::deserialize(reader)?.into();

		if version != 1 {
			return Err(Error::UnsupportedVersion(version));
		}

		let mut last_section_id = 0;

		loop {
			match Section::deserialize(reader) {
				Err(Error::UnexpectedEof) => { break; },
				Err(e) => { return Err(e) },
				Ok(section) => {
					if section.id() != 0 {
						if last_section_id > section.id() {
							return Err(Error::SectionsOutOfOrder);
						} else if last_section_id == section.id() {
							return Err(Error::DuplicatedSections(last_section_id));
						}
						last_section_id = section.id();
					}
					sections.push(section);
				}
			}
		}

		let module = Module {
			magic: LittleEndian::read_u32(&magic),
			version: version,
			sections: sections,
		};

		if module.code_section().map(|cs| cs.bodies().len()).unwrap_or(0) !=
			module.function_section().map(|fs| fs.entries().len()).unwrap_or(0)
		{
			return Err(Error::InconsistentCode);
		}

		Ok(module)
	}
}

impl Serialize for Module {
	type Error = Error;

	fn serialize<W: io::Write>(self, w: &mut W) -> Result<(), Self::Error> {
		Uint32::from(self.magic).serialize(w)?;
		Uint32::from(self.version).serialize(w)?;
		for section in self.sections.into_iter() {
			section.serialize(w)?;
		}
		Ok(())
	}
}

#[derive(Debug, Copy, Clone)]
struct PeekSection<'a> {
	cursor: usize,
	region: &'a [u8],
}

impl<'a> io::Read for PeekSection<'a> {
	fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
		let available = ::std::cmp::min(buf.len(), self.region.len() - self.cursor);
		if available < buf.len() {
			return Err(::std::io::Error::from(::std::io::ErrorKind::UnexpectedEof));
		}

		let range = self.cursor..self.cursor + buf.len();
		buf.copy_from_slice(&self.region[range]);

		self.cursor += available;
		Ok(available)
	}
}

/// Returns size of the module in the provided stream
pub fn peek_size(source: &[u8]) -> usize {
	if source.len() < 9 {
		return 0;
	}

	let mut cursor = 8;
	loop {
		let (new_cursor, section_id, section_len) = {
			let mut peek_section = PeekSection { cursor: 0, region: &source[cursor..] };
			let section_id: u8 = match super::VarUint7::deserialize(&mut peek_section) {
				Ok(res) => res.into(),
				Err(_) => { break; },
			};
			let section_len: u32 = match super::VarUint32::deserialize(&mut peek_section) {
				Ok(res) => res.into(),
				Err(_) => { break; },
			};

			(peek_section.cursor, section_id, section_len)
		};

		if section_id <= 11 && section_len > 0 {
			let next_cursor = cursor + new_cursor + section_len as usize;
			if next_cursor > source.len() {
				break;
			} else if next_cursor == source.len() {
				cursor = next_cursor;
				break;
			}
			cursor = next_cursor;
		} else {
			break;
		}
	}

	cursor
}

#[cfg(test)]
mod integration_tests {

	use super::super::{deserialize_file, serialize, deserialize_buffer, Section};
	use super::Module;

	#[test]
	fn hello() {
		let module = deserialize_file("./res/cases/v1/hello.wasm").expect("Should be deserialized");

		assert_eq!(module.version(), 1);
		assert_eq!(module.sections().len(), 8);
	}

	#[test]
	fn serde() {
		let module = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
		let buf = serialize(module).expect("serialization to succeed");

		let module_new: Module = deserialize_buffer(&buf).expect("deserialization to succeed");
		let module_old = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");

		assert_eq!(module_old.sections().len(), module_new.sections().len());
	}

	#[test]
	fn serde_type() {
		let mut module = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
		module.sections_mut().retain(|x| {
			if let &Section::Type(_) = x { true } else { false }
		});

		let buf = serialize(module).expect("serialization to succeed");

		let module_new: Module = deserialize_buffer(&buf).expect("deserialization to succeed");
		let module_old = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
		assert_eq!(
			module_old.type_section().expect("type section exists").types().len(),
			module_new.type_section().expect("type section exists").types().len(),
			"There should be equal amount of types before and after serialization"
		);
	}

	#[test]
	fn serde_import() {
		let mut module = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
		module.sections_mut().retain(|x| {
			if let &Section::Import(_) = x { true } else { false }
		});

		let buf = serialize(module).expect("serialization to succeed");

		let module_new: Module = deserialize_buffer(&buf).expect("deserialization to succeed");
		let module_old = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
		assert_eq!(
			module_old.import_section().expect("import section exists").entries().len(),
			module_new.import_section().expect("import section exists").entries().len(),
			"There should be equal amount of import entries before and after serialization"
		);
	}

	#[test]
	fn serde_code() {
		let mut module = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
		module.sections_mut().retain(|x| {
			if let &Section::Code(_) = x { return true }
			if let &Section::Function(_) = x { true } else { false }
		});

		let buf = serialize(module).expect("serialization to succeed");

		let module_new: Module = deserialize_buffer(&buf).expect("deserialization to succeed");
		let module_old = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
		assert_eq!(
			module_old.code_section().expect("code section exists").bodies().len(),
			module_new.code_section().expect("code section exists").bodies().len(),
			"There should be equal amount of function bodies before and after serialization"
		);
	}

	#[test]
	fn const_() {
		use super::super::Opcode::*;

		let module = deserialize_file("./res/cases/v1/const.wasm").expect("Should be deserialized");
		let func = &module.code_section().expect("Code section to exist").bodies()[0];
		assert_eq!(func.code().elements().len(), 20);

		assert_eq!(I64Const(9223372036854775807), func.code().elements()[0]);
		assert_eq!(I64Const(-9223372036854775808), func.code().elements()[1]);
		assert_eq!(I64Const(-1152894205662152753), func.code().elements()[2]);
		assert_eq!(I64Const(-8192), func.code().elements()[3]);
		assert_eq!(I32Const(1024), func.code().elements()[4]);
		assert_eq!(I32Const(2048), func.code().elements()[5]);
		assert_eq!(I32Const(4096), func.code().elements()[6]);
		assert_eq!(I32Const(8192), func.code().elements()[7]);
		assert_eq!(I32Const(16384), func.code().elements()[8]);
		assert_eq!(I32Const(32767), func.code().elements()[9]);
		assert_eq!(I32Const(-1024), func.code().elements()[10]);
		assert_eq!(I32Const(-2048), func.code().elements()[11]);
		assert_eq!(I32Const(-4096), func.code().elements()[12]);
		assert_eq!(I32Const(-8192), func.code().elements()[13]);
		assert_eq!(I32Const(-16384), func.code().elements()[14]);
		assert_eq!(I32Const(-32768), func.code().elements()[15]);
		assert_eq!(I32Const(-2147483648), func.code().elements()[16]);
		assert_eq!(I32Const(2147483647), func.code().elements()[17]);
	}

	#[test]
	fn store() {
		use super::super::Opcode::*;

		let module = deserialize_file("./res/cases/v1/offset.wasm").expect("Should be deserialized");
		let func = &module.code_section().expect("Code section to exist").bodies()[0];

		assert_eq!(func.code().elements().len(), 5);
		assert_eq!(I64Store(0, 32), func.code().elements()[2]);
	}

	#[test]
	fn peek() {
		use super::peek_size;

		let module = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
		let mut buf = serialize(module).expect("serialization to succeed");

		buf.extend_from_slice(&[1, 5, 12, 17]);

		assert_eq!(peek_size(&buf), buf.len() - 4);
	}


	#[test]
	fn peek_2() {
		use super::peek_size;

		let module = deserialize_file("./res/cases/v1/offset.wasm").expect("Should be deserialized");
		let mut buf = serialize(module).expect("serialization to succeed");

		buf.extend_from_slice(&[0, 0, 0, 0, 0, 1, 5, 12, 17]);

		assert_eq!(peek_size(&buf), buf.len() - 9);
	}

	#[test]
	fn peek_3() {
		use super::peek_size;

		let module = deserialize_file("./res/cases/v1/peek_sample.wasm").expect("Should be deserialized");
		let buf = serialize(module).expect("serialization to succeed");

		assert_eq!(peek_size(&buf), buf.len());
	}

	#[test]
	fn module_default_round_trip() {
		let module1 = Module::default();
		let buf = serialize(module1).expect("Serialization should succeed");

		let module2: Module = deserialize_buffer(&buf).expect("Deserialization should succeed");
		assert_eq!(Module::default().magic, module2.magic);
	}

	#[test]
	fn names() {
		use super::super::name_section::NameSection;

		let module = deserialize_file("./res/cases/v1/with_names.wasm")
			.expect("Should be deserialized")
			.parse_names()
			.expect("Names to be parsed");

		let mut found_section = false;
		for section in module.sections() {
			match *section {
				Section::Name(ref name_section) => {
					match *name_section {
						NameSection::Function(ref function_name_section) => {
							assert_eq!(
								function_name_section.names().get(0).expect("Should be entry #0"),
								"elog"
							);
							assert_eq!(
								function_name_section.names().get(11).expect("Should be entry #0"),
								"_ZN48_$LT$pwasm_token_contract..Endpoint$LT$T$GT$$GT$3new17hc3ace6dea0978cd9E"
							);

							found_section = true;
						},
						_ => {},
					}
				},
				_ => {},
			}
		}

		assert!(found_section, "Name section should be present in dedicated example");
	}

	#[test]
	#[should_panic]
	fn wrong_varuint1_case() {
		let _module = deserialize_file("./res/cases/v1/varuint1_1.wasm")
			.expect("Maybe shouldn't be deserialized");
	}


	#[test]
	fn memory_space() {
		let module = deserialize_file("./res/cases/v1/two-mems.wasm").expect("failed to deserialize");
		assert_eq!(module.memory_space(), 2);
	}
}
