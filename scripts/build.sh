#!/bin/bash
rm -rf pkg
rm -rf ../../../modules/companion-module-levandevideo-vmix
yarn install && yarn build && yarn companion-module-build
cp -r pkg ../../../modules/companion-module-levandevideo-vmix