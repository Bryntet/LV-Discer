#!/usr/bin/env nu
rm -rf ../../../modules/companion-module-levandevideo-vmix;
tsc;
yarn companion-module-build; cp -r ./pkg ../../../modules/companion-module-levandevideo-vmix;