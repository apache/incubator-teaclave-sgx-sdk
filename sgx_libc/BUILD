load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library")

rust_library(
    name = "sgx_libc",
    srcs = glob([
        "src/*.rs",
        "src/linux/*.rs",
        "src/linux/x86_64/*.rs",
        "src/android/*.rs",
        "src/andorid/aarch64/*.rs",
    ]),
    crate_features = ["align"],
    edition = "2018",
    visibility = ["//visibility:public"],
    deps = ["//sgx_types"],
)
