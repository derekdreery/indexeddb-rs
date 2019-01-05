#!/bin/sh

set -ex
pushd "$(dirname $0)"

cargo build --target wasm32-unknown-unknown
wasm-bindgen ../target/wasm32-unknown-unknown/debug/indexeddb_test.wasm --out-dir=. --no-typescript

npm install
npm run serve

popd
