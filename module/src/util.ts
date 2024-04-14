import { DropdownChoice, InstanceBase } from "@companion-module/base";
import { FlipUpVMixCoordinator } from "../built/rust_pkg/rust_wasm_test_edvin";


export interface InstanceBaseExt<TConfig> extends InstanceBase<TConfig> {
	config: TConfig
    rust_main: FlipUpVMixCoordinator
    focused_players: DropdownChoice[]
    foc_player_ind: number
}