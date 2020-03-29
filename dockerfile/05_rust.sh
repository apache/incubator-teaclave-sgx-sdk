cd /root && \
curl 'https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init' --output /root/rustup-init && \
chmod +x /root/rustup-init && \
echo '1' | /root/rustup-init --default-toolchain ${rust_toolchain} && \
echo 'source /root/.cargo/env' >> /root/.bashrc && \
/root/.cargo/bin/rustup component add rust-src rls rust-analysis clippy rustfmt && \
/root/.cargo/bin/cargo install xargo && \
rm /root/rustup-init && rm -rf /root/.cargo/registry && rm -rf /root/.cargo/git
