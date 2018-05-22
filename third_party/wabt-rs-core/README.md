# WABT bindings for Rust

[![crates.io](https://img.shields.io/crates/v/wabt.svg)](https://crates.io/crates/wabt)
[![docs.rs](https://docs.rs/wabt/badge.svg)](https://docs.rs/wabt/)

Rust bindings for [WABT](https://github.com/WebAssembly/wabt). Work in progress.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
wabt = "0.2"
```

## Example

`wat2wasm` (previously known as `wast2wasm`):

```rust
extern crate wabt;
use wabt::wat2wasm;

fn main() {
    assert_eq!(
        wat2wasm("(module)").unwrap(),
        &[
            0, 97, 115, 109, // \0ASM - magic
            1, 0, 0, 0       //  0x01 - version
        ]
    );
}
```

`wasm2wat`:

```rust
extern crate wabt;
use wabt::wasm2wat;
fn main() {
    assert_eq!(
        wasm2wat(&[
            0, 97, 115, 109, // \0ASM - magic
            1, 0, 0, 0       //    01 - version
        ]),
        Ok("(module)\n".to_owned()),
    );
}
```

`wabt` can be also used for parsing the official [testsuite](https://github.com/WebAssembly/testsuite) scripts.

```rust
use wabt::script::{ScriptParser, Command, CommandKind, Action, Value};
 
let wast = r#"
;; Define anonymous module with function export named `sub`.
(module 
  (func (export "sub") (param $x i32) (param $y i32) (result i32)
    ;; return x - y;
    (i32.sub
      (get_local $x) (get_local $y)
    )
  )
)
 
;; Assert that invoking export `sub` with parameters (8, 3)
;; should return 5.
(assert_return
  (invoke "sub"
    (i32.const 8) (i32.const 3)
  )
  (i32.const 5)
)
"#;
 
let mut parser = ScriptParser::from_str(wast)?;
while let Some(Command { kind, .. }) = parser.next()? { 
    match kind {
        CommandKind::Module { module, name } => {
            // The module is declared as annonymous.
            assert_eq!(name, None);
 
            // Convert the module into the binary representation and check the magic number.
            let module_binary = module.into_vec()?;
            assert_eq!(&module_binary[0..4], &[0, 97, 115, 109]);
        }
        CommandKind::AssertReturn { action, expected } => {
            assert_eq!(action, Action::Invoke { 
                module: None,
                field: "sub".to_string(),
                args: vec![
                    Value::I32(8),
                    Value::I32(3)
                ],
            });
            assert_eq!(expected, vec![Value::I32(5)]);
        },
        _ => panic!("there are no other commands apart from that defined above"),
    }
}
```
