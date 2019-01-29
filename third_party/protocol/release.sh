#! /bin/sh -ea

cd protocol-derive
cargo publish

cd ../protocol
cargo publish

echo "pls create and push a tag to Git"

