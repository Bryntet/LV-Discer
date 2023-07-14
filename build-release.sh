#!/bin/bash

wasm-pack build --target nodejs --release -d rust_pkg && rm node_modules/rust-wasm-test-edvin -rf && fnm use 18 && yarn install --check-files && yarn companion-module-build
