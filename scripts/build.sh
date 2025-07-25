#!/bin/bash
rm -rf pkg
rm -rf ../../../modules/companion-module-levandevideo-vmix
yarn install --check-files && tsc --build tsconfig.json && yarn companion-module-build --dev
cp -r pkg ../../../modules/companion-module-levandevideo-vmix