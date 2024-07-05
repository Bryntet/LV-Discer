import { DropdownChoice, InstanceBase } from "@companion-module/base";
import {ApiClient} from "./coordinator_communication";


export interface InstanceBaseExt<TConfig> extends InstanceBase<TConfig> {
	config: TConfig
    coordinator: ApiClient
    focused_players: DropdownChoice[]
    foc_player_id: string
}