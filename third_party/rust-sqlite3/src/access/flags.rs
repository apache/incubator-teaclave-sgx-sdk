use std::default::Default;
use libc::c_int;

/// These bit values are intended for use in the
/// 3rd parameter to the [sqlite3_open_v2()] interface
bitflags!(
  pub struct Flags: c_int {
    const OPEN_READONLY       = 0x00000001;
    const OPEN_READWRITE      = 0x00000002;
    const OPEN_CREATE         = 0x00000004;
    const OPEN_URI            = 0x00000040;
    const OPEN_MEMORY         = 0x00000080;
    const OPEN_NOMUTEX        = 0x00008000;
    const OPEN_FULLMUTEX      = 0x00010000;
    const OPEN_SHAREDCACHE    = 0x00020000;
    const OPEN_PRIVATECACHE   = 0x00040000;
  }
);

impl Default for Flags {
    fn default() -> Flags{Flags::OPEN_READWRITE | Flags::OPEN_CREATE | Flags::OPEN_NOMUTEX}
}
