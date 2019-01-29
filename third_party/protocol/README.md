# protocol

[![Build Status](https://travis-ci.org/dylanmckay/protocol.svg?branch=master)](https://travis-ci.org/dylanmckay/protocol)
[![Crates.io](https://img.shields.io/crates/v/protocol.svg)](https://crates.io/crates/protocol)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

[Documentation](https://docs.rs/protocol)

Easy protocol definitions in Rust.

This crate adds a custom derive that can be added to types, allowing
structured data to be sent and received from any IO stream. `macro_rules` macros also exist which can be used to define sendable/receivable types too.

Networking is built-in, with special support for TCP and UDP.

The protocol you define can be used outside of networking too - see the `Parcel::from_raw_bytes` and `Parcel::raw_bytes` methods.

This crate also provides:

* [TCP](https://docs.rs/protocol/0.3.4/protocol/wire/stream/index.html) and [UDP](https://docs.rs/protocol/0.3.4/protocol/wire/dgram/index.html) modules for easy sending and receiving of `Parcel`s
* A generic [middleware](https://docs.rs/protocol/0.3.4/protocol/wire/middleware/index.html) library for automatic transformation of sent/received data
  * Middleware has already been written to support [compression](https://docs.rs/protocol/0.3.4/protocol/wire/middleware/compression/index.html)
  * Custom middleware can be implemented via a trait with two methods

Checkout the [examples](./examples) folder for usage.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
protocol = "0.3"
protocol-derive = "0.3"
```

## Under the hood

The most interesting part here is the [`protocol::Parcel`](https://docs.rs/protocol/0.3.4/protocol/trait.Parcel.html) trait. Any type that implements this trait can then be serialized to and from a byte stream. All primitive types, standard collections, tuples, and arrays implement this trait.

This crate becomes particularly useful when you define your own `Parcel` types. You can use `#[derive(Protocol)]` to do this, or you can use the `define_composite_type!` macro instead. Note that in order for a type to implement `Parcel`, it must also implement `Clone`, `Debug`, and `PartialEq`.

```rust
#[derive(Parcel, Clone, Debug, PartialEq]
pub struct Health(f32);

#[derive(Parcel, Clone, Debug, PartialEq]
pub struct SetPlayerPosition {
    pub position: (f32, f32),
    pub health: Health,
    pub values: Vec<String>,
}
```

### Custom derive

Any user-defined type can have the `Parcel` trait automatically derived.

## Example

```rust
#[macro_use] extern crate protocol_derive;
#[macro_use] extern crate protocol;

#[derive(Protocol, Clone, Debug, PartialEq)]
pub struct Handshake;

#[derive(Protocol, Clone, Debug, PartialEq)]
pub struct Hello {
    id: i64,
    data: Vec<u8>,
}

#[derive(Protocol, Clone, Debug, PartialEq)]
pub struct Goodbye {
    id: i64,
    reason: String,
}

#[derive(Protocol, Clone, Debug, PartialEq)]
pub struct Node {
    name: String,
    enabled: bool
}

// Defines a packet kind enum.
define_packet_kind!(Packet: u32 {
    0x00 => Handshake,
    0x01 => Hello,
    0x02 => Goodbye
});

fn main() {
    use std::net::TcpStream;

    let stream = TcpStream::connect("127.0.0.1:34254").unwrap();
    let mut connection = protocol::wire::stream::Connection::new(stream, protocol::wire::middleware::pipeline::default());

    connection.send_packet(&Packet::Handshake(Handshake)).unwrap();
    connection.send_packet(&Packet::Hello(Hello { id: 0, data: vec![ 55 ]})).unwrap();
    connection.send_packet(&Packet::Goodbye(Goodbye { id: 0, reason: "leaving".to_string() })).unwrap();

    loop {
        if let Some(response) = connection.receive_packet().unwrap() {
            println!("{:?}", response);
            break;
        }
    }
}
```

## Enums

### Discriminators

Enum values can be transmitted either by their 1-based variant index, or by transmitting the string name of each variant.

**NOTE:** The default behaviour is to use *the 1-based variant index* (`integer`).

This behaviour can be changed by the `#[protocol(discriminant = "string")]` attribute.

Supported discriminant types:

* `integer` (default)
    * This transmits the 1-based variant number as the over-the-wire discriminant
    * Enum variants cannot be reordered in the source without breaking the protocol
* `string`
    * This transmits the enum variant name as the over-the-wire discriminant
    * This uses more bytes per message, but it very flexible

**N.B.** `string` should become the default discriminant type for `#[derive(Protocol)]`. This would be a breaking change.
Perhaps a major release?

```rust
#[derive(Protocol, Clone, Debug, PartialEq)]
#[protocol(discriminant = "string")]
pub enum PlayerState {
  Stationary,
  Flying { velocity: (f32,f32,f32) },
  Jumping { height: f32 },
}
```

### Misc

You can rename the variant for their serialisation.

```rust
#[derive(Protocol, Clone, Debug, PartialEq)]
#[protocol(discriminant = "string")]
pub enum Foo {
  Bar,
  #[protocol(name = "Biz")] // the Bing variant will be send/received as 'Biz'.
  Bing,
  Baz,
}
```
