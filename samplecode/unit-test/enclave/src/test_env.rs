use std::env::*;
use std::path::Path;

pub  fn test_env_self_exe_path() {
    let path = current_exe();
    assert!(path.is_ok());
    let path = path.unwrap();

    // Hard to test this function
    assert!(path.is_absolute());
}

pub fn test_env_current_dir() {
    assert!((!Path::new("test-path").is_absolute()));
    println!("{:?}", current_dir().unwrap());
}

pub fn test_env_home_dir() {
    let dir = home_dir();
    println!("[ENV] HOME: {:?}", dir.unwrap());
}
