use wabt::script::{ScriptParser, Command, CommandKind};
use parity_wasm::elements::{Module, deserialize_buffer};

pub fn spec(path: &str) {
	let mut parser = ScriptParser::from_file(&format!("./testsuite/{}.wast", path)).expect("Can't read spec script");
	while let Some(Command { kind, line }) = parser.next().expect("Failed to iterate") {
		match kind {
			CommandKind::AssertMalformed { module, .. } =>
			{
				match deserialize_buffer::<Module>(
					&module.into_vec().expect("Invalid filename provided")
				) {
					Ok(_) => panic!("Expected invalid module definition, got some module!"),
					Err(e) => println!("assert_invalid at line {} - success ({:?})", line, e),
				}
			}
			CommandKind::Module { module, .. } => {
				match deserialize_buffer::<Module>(
					&module.into_vec().expect("Invalid filename provided")
				) {
					Ok(_) => println!("module at line {} - parsed ok", line),
					Err(e) => panic!("Valid module reported error ({:?})", e),
				}
			}
			_ => {
				// Skipping interpreted
			}
		}
	}
}