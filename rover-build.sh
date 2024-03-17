#!/bin/bash

rm -rf pkg
rm -rf ../../modules/companion-module-levandevideo-vmix
rm node_modules/rust-wasm-test-edvin -rf & rm pkg.tgz
yarn companion-module-build && cp -r pkg ../../modules/companion-module-levandevideo-vmix