pub mod test {
    #[cfg(feature = "enclave")]
    pub fn ocall() {}

    #[cfg(feature = "app")]
    pub fn entry() {}
}
