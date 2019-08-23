#![deny(clippy::all)]
#![no_std]

extern crate alloc;

pub fn lookup_digits_mod_at_position(x: u8, q: u8, pos: usize) -> &'static [u8] {
    unsafe {
        let tab = c_get_table(q, pos);
        let len = c_num_digits(q, pos);
        alloc::slice::from_raw_parts(tab.add(len*(x as usize)), len)
    }
}

pub fn lookup_defined_for_mod(q: u8) -> bool {
    unsafe {
        c_num_digits(q, 0) > 0
    }
}

extern {
    fn c_get_table(q: u8, pos: usize) -> *const u8;
    fn c_num_digits(q: u8, pos: usize) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup() {
        if lookup_defined_for_mod(3) {
            assert_eq!(lookup_digits_mod_at_position(0,3,0), &[ 0,0,0,0,0,0 ]);
            assert_eq!(lookup_digits_mod_at_position(2,3,6).to_vec(), vec![ 2,0,1,1,2,0,1,1,2,2,0,2,1,1,0,0,2,0,1,1,1,0,2,0,1,1,2,1,0,2,2,0,0,0,0,0 ]);
        }
    }
}
