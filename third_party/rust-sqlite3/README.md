# Rust-Sqlite3

Rustic bindings for sqlite3. *OBSOLETE in favor of [jgallagher/rusqlite](https://github.com/jgallagher/rusqlite)*

Copyright (c) 2014-2017 [Dan Connolly][dckc] and contributors

Share and enjoy. LICENSE: MIT.

[dckc]: http://www.madmode.com/


## Documentation, Status

[View the documentation online][docs]

Three layers of API are provided:

  - `mod ffi` provides exhaustive, though unsafe, [bindgen] bindings for `libsqlite.h`
  - `mod core` provides a minimal safe interface to the basic sqlite3 API
  - `mod types` provides `ToSql`/`FromSql` traits, and the library provides
    convenient `query()` and `update()` APIs.
  
The API design is perhaps stabilizing, though testing is uneven and I
have not used the library beyond trivial integration tests.

[docs]: http://dckc.github.io/rust-sqlite3
[![Build Status](https://travis-ci.org/dckc/rust-sqlite3.svg?branch=master)](https://travis-ci.org/dckc/rust-sqlite3)

### TODO

  - another thorough read-through of the sqlite API intro,
    with unit tests to match; especially...
    - unit testing other than the happy-paths
  - `ToSql`/`FromSql` can now be implemented by clients,
    but the `types` module probably doesn't hit the 80% mark yet;
    e.g. it's missing uint and &[u8].
  - investigate test coverage tools for rust
  - basic benchmarking


## Motivation and Acknowledgements

I was looking into [sandstorm][], a personal cloud platform with an
architecture based on the wonderful [capability security][capsec]
paradigm, and I found a rust application, [acronymy][], that uses the
native API rather than the traditional POSIX environment.

[sandstorm]: https://sandstorm.io/
[capsec]: http://www.erights.org/elib/capability/ode/ode-capabilities.html
[acronymy]: https://github.com/dwrensha/acronymy
[bindgen]: https://github.com/crabtw/rust-bindgen

I started poring over the code and followed the dependency link to
linuxfood's [rustsqlite][]. I started working on a [memory safety
issue][92] etc. but soon found a number of large-scale API design
issues that I wasn't sure how to approach with the upstream
developers. I was also inspired by `FromSql`, `ToSql` and such
from sfackler's [rust-postgres] API.

So I started from scratch, using [bindgen][], `Result` (sum types) etc.

[rustsqlite]: https://github.com/linuxfood/rustsqlite
[92]: https://github.com/linuxfood/rustsqlite/issues/92
[rust-postgres]: https://github.com/sfackler/rust-postgres
[bindgen]: https://github.com/crabtw/rust-bindgen
