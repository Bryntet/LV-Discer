#!/bin/bash

wasm-pack build --target nodejs --release -d pkg && rm node_modules/rust-wasm-test-edvin -rf && yarn install --check-files
