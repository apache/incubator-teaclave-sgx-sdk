load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library")

rust_library(
    name = "sgx_trts",
    srcs = glob(["src/*.rs"]),
    edition = "2021",
    visibility = ["//visibility:public"],
    deps = [
        "//sgx_libc",
        "//sgx_types",
    ],
)
