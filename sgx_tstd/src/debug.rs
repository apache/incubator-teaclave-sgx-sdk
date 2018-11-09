#[no_mangle]
pub extern "C" fn __assert_fail(
    __assertion: *const u8,
    __file: *const u8,
    __line: u32,
    __function: *const u8,
) -> ! {
    use core::intrinsics::abort;
    unsafe { abort() }
}
