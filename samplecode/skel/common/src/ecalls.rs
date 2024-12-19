pub mod foo {
    #[cfg(feature = "app")]
    pub fn ecall() {
        todo!()
    }

    #[cfg(feature = "enclave")]
    pub fn entry() {
        todo!()
    }
}
