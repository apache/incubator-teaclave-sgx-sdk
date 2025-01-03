mod arg;
mod ecall;
mod ocall;

pub use arg::{In, Out, Update};
pub use ecall::{untrust_ecall, Ecall, EcallWrapper};
pub use ocall::{OTabEntry, OcallTable};
pub use sgx_edl_macros::{ecall, ecalls};
pub use sgx_types::error::SgxStatus;

//#[macro_export]
//macro_rules! ecalls_set_idx {
//    // 基础情况：当只有一个 ident 时
//    ($last:ident) => {
//        #[allow(non_upper_case_globals)]
//        pub const $last: usize = 0usize;
//    };
//    // 递归情况：当有多个 idents 时
//    ($first:ident, $($rest:ident),+) => {
//        #[allow(non_upper_case_globals)]
//        pub const $first: usize = 0usize;
//        // 辅助宏用于处理后续的 idents 和增加计数
//        $crate::ecalls_set_idx!(@internal 1; $($rest),+);
//    };
//    // 内部辅助宏用于处理除了第一个之外的所有 idents
//    (@internal $n:expr; $current:ident) => {
//        #[allow(non_upper_case_globals)]
//        pub const $current: usize = $n;
//    };
//    (@internal $n:expr; $current:ident, $($rest:ident),+) => {
//        #[allow(non_upper_case_globals)]
//        pub const $current: usize = $n;
//        $crate::ecalls_set_idx!(@internal $n + 1usize; $($rest),+);
//    };
//}
//
//#[macro_export]
//macro_rules! ecall_table {
//    ($($f:ident), *) => {
//        pub static E_TABLE: &'static [fn(*const u8)] = &[
//            $($f::entry),
//            *
//        ];
//    };
//}
//
//#[macro_export]
//macro_rules! ecalls {
//    [$($f:tt)*] => {
//        $crate::ecall_table!($($f)*);
//
//        mod idx {
//            $crate::ecalls_set_idx!($($f)*);
//        }
//    };
//}
//
//#[macro_export]
//macro_rules! ecalls_new {
//    // 使用 tt 匹配整个函数签名
//    (@ $signature:tt) => {
//        println!("匹配到函数签名: {}", stringify!($signature));
//    };
//
//    // 匹配其他情况
//    ($other:tt) => {
//        println!("未匹配到函数签名: {}", stringify!($other));
//    };
//}

impl Update for String {
    fn update(&mut self, other: &Self) {}
}
