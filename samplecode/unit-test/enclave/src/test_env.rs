use std::env;

pub fn env_tests() {
    let p = env::vars_os();
    assert_ne!(0, p.size_hint().0);
}
