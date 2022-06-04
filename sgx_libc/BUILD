load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library")

rust_library(
    name = "sgx_libc",
    srcs = glob(["src/*.rs"]),
    crate_features = ["align"],
    edition = "2021",
    visibility = ["//visibility:public"],
    deps = ["//sgx_types"],
)
