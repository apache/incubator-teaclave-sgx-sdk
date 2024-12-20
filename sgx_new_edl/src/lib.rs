use args::{In, Out, Update};

pub mod args;
pub mod ecall;
pub mod ocall;

use ecall::EcallWrapper;
pub use sgx_edl_macros::ecall;

#[macro_export]
macro_rules! ecalls_set_idx {
    // 基础情况：当只有一个 ident 时
    ($last:ident) => {
        #[allow(non_upper_case_globals)]
        pub const $last: usize = 0usize;
    };
    // 递归情况：当有多个 idents 时
    ($first:ident, $($rest:ident),+) => {
        #[allow(non_upper_case_globals)]
        pub const $first: usize = 0usize;
        // 辅助宏用于处理后续的 idents 和增加计数
        $crate::ecalls_set_idx!(@internal 1; $($rest),+);
    };
    // 内部辅助宏用于处理除了第一个之外的所有 idents
    (@internal $n:expr; $current:ident) => {
        #[allow(non_upper_case_globals)]
        pub const $current: usize = $n;
    };
    (@internal $n:expr; $current:ident, $($rest:ident),+) => {
        #[allow(non_upper_case_globals)]
        pub const $current: usize = $n;
        $crate::ecalls_set_idx!(@internal $n + 1usize; $($rest),+);
    };
}

#[macro_export]
macro_rules! ecall_table {
    ($($f:ident), *) => {
        pub static E_TABLE: &'static [fn(*const u8)] = &[
            $($f::entry),
            *
        ];
    };
}

#[macro_export]
macro_rules! ecalls {
    [$($f:tt)*] => {
        $crate::ecall_table!($($f)*);

        mod idx {
            $crate::ecalls_set_idx!($($f)*);
        }
    };
}

pub mod ecalls {
    use crate::ecall::ETabEntry;
    pub const TABLE: &[ETabEntry] = &[];
}

impl Update for String {
    fn update(&mut self, other: &Self) {}
}

ecalls!(foo);

mod foo {
    use super::*;

    struct _PhantomMarker<'a> {
        a: &'a (),
    }

    impl<'a> ecall::Ecall<String> for _PhantomMarker<'a> {
        const IDX: usize = idx::foo;
        type Args = (In<'a, String>);

        fn call(&self, args: Self::Args) {
            todo!()
        }
    }

    pub fn ecall<'a>(eid: usize, a1: In<'a, String>, a2: Out<'a, String>) {
        ecall::EcallWrapper::wrapper_u(&_PhantomMarker { a: &() }, eid, todo!(), (a1));
    }

    pub fn entry(data: *const u8) {
        ecall::EcallWrapper::wrapper_t(&_PhantomMarker { a: &() }, data);
    }
}
