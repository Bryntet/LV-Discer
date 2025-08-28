#!/bin/bash
rm -rf pkg
rm -rf ../../../modules/companion-module-levandevideo-vmix
cd ../rust_controller
wasm-pack build --target nodejs --release -d ../module/built/rust_pkg
cd ../module
rm node_modules/rust-wasm-test-edvin -rf & rm pkg.tgz & rm lv-vmix.zip
yarn install
yarn companion-module-build && cp -r ./pkg ../../../modules/companion-module-levandevideo-vmix
7z a lv-vmix.zip ./pkg/*

