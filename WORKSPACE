load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")

##############################################
#                                            #
#                   Rust                     #
#                                            #
##############################################
http_archive(
    name = "io_bazel_rules_rust",
    sha256 = "29d9fc1cdbd737c51db5983d1ac8e64cdc684c4683bafbcc624d3d81de92a32f",
    strip_prefix = "rules_rust-8417c8954efbd0cefc8dd84517b2afff5e907d5a",
    urls = [
        "https://github.com/bazelbuild/rules_rust/archive/8417c8954efbd0cefc8dd84517b2afff5e907d5a.tar.gz",
    ],
)

http_archive(
    name = "bazel_skylib",
    sha256 = "eb5c57e4c12e68c0c20bc774bfbc60a568e800d025557bc4ea022c6479acc867",
    strip_prefix = "bazel-skylib-0.6.0",
    url = "https://github.com/bazelbuild/bazel-skylib/archive/0.6.0.tar.gz",
)

load("@io_bazel_rules_rust//rust:repositories.bzl", "rust_repository_set")

rust_repository_set(
    name = "rust_linux_x86_64",
    exec_triple = "x86_64-unknown-linux-gnu",
    extra_target_triples = [],
    iso_date = "2019-05-22",
    version = "nightly",
)

load("@io_bazel_rules_rust//:workspace.bzl", "bazel_version")

bazel_version(name = "bazel_version")