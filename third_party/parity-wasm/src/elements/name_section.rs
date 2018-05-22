use std::prelude::v1::*;
use std::io::{Read, Write};

use super::{Deserialize, Error, Module, Serialize, VarUint32, VarUint7, Type};
use super::index_map::IndexMap;

const NAME_TYPE_MODULE: u8 = 0;
const NAME_TYPE_FUNCTION: u8 = 1;
const NAME_TYPE_LOCAL: u8 = 2;

/// Debug name information.
#[derive(Clone, Debug, PartialEq)]
pub enum NameSection {
	/// Module name section.
	Module(ModuleNameSection),

	/// Function name section.
	Function(FunctionNameSection),

	/// Local name section.
	Local(LocalNameSection),

	/// Name section is unparsed.
	Unparsed {
		/// The numeric identifier for this name section type.
		name_type: u8,
		/// The contents of this name section, unparsed.
		name_payload: Vec<u8>,
	},
}

impl NameSection {
	/// Deserialize a name section.
	pub fn deserialize<R: Read>(
		module: &Module,
		rdr: &mut R,
	) -> Result<NameSection, Error> {
		let name_type: u8 = VarUint7::deserialize(rdr)?.into();
		let name_payload_len: u32 = VarUint32::deserialize(rdr)?.into();
		let name_section = match name_type {
			NAME_TYPE_MODULE => NameSection::Module(ModuleNameSection::deserialize(rdr)?),
			NAME_TYPE_FUNCTION => NameSection::Function(FunctionNameSection::deserialize(module, rdr)?),
			NAME_TYPE_LOCAL => NameSection::Local(LocalNameSection::deserialize(module, rdr)?),
			_ => {
				let mut name_payload = vec![0u8; name_payload_len as usize];
				rdr.read_exact(&mut name_payload)?;
				NameSection::Unparsed {
					name_type,
					name_payload,
				}
			}
		};
		Ok(name_section)
	}
}

impl Serialize for NameSection {
	type Error = Error;

	fn serialize<W: Write>(self, wtr: &mut W) -> Result<(), Error> {
		let (name_type, name_payload) = match self {
			NameSection::Module(mod_name) => {
				let mut buffer = vec![];
				mod_name.serialize(&mut buffer)?;
				(NAME_TYPE_MODULE, buffer)
			}
			NameSection::Function(fn_names) => {
				let mut buffer = vec![];
				fn_names.serialize(&mut buffer)?;
				(NAME_TYPE_FUNCTION, buffer)
			}
			NameSection::Local(local_names) => {
				let mut buffer = vec![];
				local_names.serialize(&mut buffer)?;
				(NAME_TYPE_LOCAL, buffer)
			}
			NameSection::Unparsed {
				name_type,
				name_payload,
			} => (name_type, name_payload),
		};
		VarUint7::from(name_type).serialize(wtr)?;
		VarUint32::from(name_payload.len()).serialize(wtr)?;
		wtr.write_all(&name_payload)?;
		Ok(())
	}
}

/// The name of this module.
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleNameSection {
	name: String,
}

impl ModuleNameSection {
	/// Create a new module name section with the specified name.
	pub fn new<S: Into<String>>(name: S) -> ModuleNameSection {
		ModuleNameSection { name: name.into() }
	}

	/// The name of this module.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// The name of this module (mutable).
	pub fn name_mut(&mut self) -> &mut String {
		&mut self.name
	}
}

impl Serialize for ModuleNameSection {
	type Error = Error;

	fn serialize<W: Write>(self, wtr: &mut W) -> Result<(), Error> {
		self.name.serialize(wtr)
	}
}

impl Deserialize for ModuleNameSection {
	type Error = Error;

	fn deserialize<R: Read>(rdr: &mut R) -> Result<ModuleNameSection, Error> {
		let name = String::deserialize(rdr)?;
		Ok(ModuleNameSection { name })
	}
}

/// The names of the functions in this module.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FunctionNameSection {
	names: NameMap,
}

impl FunctionNameSection {
	/// A map from function indices to names.
	pub fn names(&self) -> &NameMap {
		&self.names
	}

	/// A map from function indices to names (mutable).
	pub fn names_mut(&mut self) -> &mut NameMap {
		&mut self.names
	}

	/// Deserialize names, making sure that all names correspond to functions.
	pub fn deserialize<R: Read>(
		module: &Module,
		rdr: &mut R,
	) -> Result<FunctionNameSection, Error> {
		let names = IndexMap::deserialize(module.functions_space(), rdr)?;
		Ok(FunctionNameSection { names })
	}
}

impl Serialize for FunctionNameSection {
	type Error = Error;

	fn serialize<W: Write>(self, wtr: &mut W) -> Result<(), Error> {
		self.names.serialize(wtr)
	}
}

/// The names of the local variables in this module's functions.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LocalNameSection {
	local_names: IndexMap<NameMap>,
}

impl LocalNameSection {
	/// A map from function indices to a map from variables indices to names.
	pub fn local_names(&self) -> &IndexMap<NameMap> {
		&self.local_names
	}

	/// A map from function indices to a map from variables indices to names
	/// (mutable).
	pub fn local_names_mut(&mut self) -> &mut IndexMap<NameMap> {
		&mut self.local_names
	}

	/// Deserialize names, making sure that all names correspond to local
	/// variables.
	pub fn deserialize<R: Read>(
		module: &Module,
		rdr: &mut R,
	) -> Result<LocalNameSection, Error> {
		let funcs = module.function_section().ok_or_else(|| {
			Error::Other("cannot deserialize local names without a function section")
		})?;
		let max_entry_space = funcs.entries().len();

		let max_signature_args = module
			.type_section()
			.map(|ts|
				ts.types()
					.iter()
					.map(|x| { let Type::Function(ref func) = *x; func.params().len() })
					.max()
					.unwrap_or(0))
			.unwrap_or(0);

		let max_locals = module
			.code_section()
			.map(|cs| cs.bodies().iter().map(|f| f.locals().len()).max().unwrap_or(0))
			.unwrap_or(0);

		let max_space = max_signature_args + max_locals;

		let deserialize_locals = |_: u32, rdr: &mut R| IndexMap::deserialize(max_space, rdr);

		let local_names = IndexMap::deserialize_with(
			max_entry_space,
			&deserialize_locals,
			rdr,
		)?;
		Ok(LocalNameSection { local_names })
	}}

impl Serialize for LocalNameSection {
	type Error = Error;

	fn serialize<W: Write>(self, wtr: &mut W) -> Result<(), Error> {
		self.local_names.serialize(wtr)
	}
}

/// A map from indices to names.
pub type NameMap = IndexMap<String>;

#[cfg(test)]
mod tests {
	use super::*;

	// A helper funtion for the tests. Serialize a section, deserialize it,
	// and make sure it matches the original.
	fn serialize_test(original: NameSection) -> Vec<u8> {
		let mut buffer = vec![];
		original
			.serialize(&mut buffer)
			.expect("serialize error");
		buffer
	}

	#[test]
	fn serialize_module_name() {
		let original = NameSection::Module(ModuleNameSection::new("my_mod"));
		serialize_test(original.clone());
	}

	#[test]
	fn serialize_function_names() {
		let mut sect = FunctionNameSection::default();
		sect.names_mut().insert(0, "hello_world".to_string());
		serialize_test(NameSection::Function(sect));
	}

	#[test]
	fn serialize_local_names() {
		let mut sect = LocalNameSection::default();
		let mut locals = NameMap::default();
		locals.insert(0, "msg".to_string());
		sect.local_names_mut().insert(0, locals);
		serialize_test(NameSection::Local(sect));
	}

	#[test]
	fn serialize_and_deserialize_unparsed() {
		let original = NameSection::Unparsed {
			// A made-up name section type which is unlikely to be allocated
			// soon, in order to allow us to test `Unparsed`.
			name_type: 120,
			name_payload: vec![0u8, 1, 2],
		};
		serialize_test(original.clone());
	}
}
