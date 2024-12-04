use args::{In, Out, Update};

pub mod args;
pub mod ecall;
pub mod ocall;

pub mod ecalls {
    use crate::ecall::ETabEntry;

    pub const TABLE: &[ETabEntry] = &[];
}

impl Update for String {
    fn update(&mut self, other: &Self) {}
}

fn efoo(a1: In<'_, String>, o1: Out<'_, String>) {
    todo!()
}

pub mod efoo {
    use super::*;
    use ecall::{Ecall, EcallWrapper};
    use ocall::OTabEntry;

    #[derive(Default)]
    struct efoo_t<'a> {
        _phantom: std::marker::PhantomData<&'a ()>,
    }

    impl<'a> Ecall for efoo_t<'a> {
        const IDX: usize = 0;
        type Args = (In<'a, String>, Out<'a, String>);

        fn call(&self, args: Self::Args) -> Self::Args {
            todo!()
        }
    }

    pub fn ecall(eid: usize, o_tab: &[OTabEntry], b: In<'_, String>, o: Out<'_, String>) {
        EcallWrapper::wrapper_u(&efoo_t::default(), eid, o_tab, (b, o));
    }

    pub fn entry(args: *const u8) {
        EcallWrapper::wrapper_t(&efoo_t::default(), args);
    }
}
