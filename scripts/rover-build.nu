#!/usr/bin/env nu
source /home/brynte/.config/nushell/env.nu
rm -rf ../../../modules/companion-module-levandevideo-vmix
yarn companion-module-build; cp -r ./pkg ../../../modules/companion-module-levandevideo-vmix