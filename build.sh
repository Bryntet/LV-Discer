#!/bin/bash
rm -rf pkg
rm -rf ../../modules/companion-module-levandevideo-vmix
wasm-pack build --target nodejs --release -d built/rust_pkg 
rm node_modules/rust-wasm-test-edvin -rf
fnm use 18 && yarn install --check-files && tsc --build tsconfig.json && yarn companion-module-build --dev 
cp -r pkg ../../modules/companion-module-levandevideo-vmix