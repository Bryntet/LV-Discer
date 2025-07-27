import {DropdownChoice, InstanceBase} from "@companion-module/base";
import {ApiClient} from "./coordinator_communication.js";


export interface InstanceBaseExt<TConfig> extends InstanceBase<TConfig> {
    config: TConfig
    coordinator: ApiClient
    focused_players: DropdownChoice[]
    foc_player_ind: number
}