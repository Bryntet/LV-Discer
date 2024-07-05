import {CompanionActionDefinitions, CompanionActionEvent} from "@companion-module/base";
import { Config } from "./config";
import { InstanceBaseExt } from "./util";
import { CompanionCommonCallbackContext } from "@companion-module/base/dist/module-api/common";
import {Player} from "./coordinator_communication";




export enum ActionId {
	LeaderboardUpdate = 'leaderboard_update',
	IncreaseScore = 'increase_score',
	RevertScoreIncrease = 'revert_score_increase',
    ResetScore = 'reset_score',
    ChangeFocusedPlayer = 'change_focused_player',
    IncreaseThrow = 'increase_throw',
    DecreaseThrow = 'decrease_throw',
    OB = 'ob',
    RunAnimation = 'run_animation',
    SetHoleInfo = 'set_hole_info',
    DoOtherLeaderboard = 'do_other_leaderboard',
}

async function parseAuto(context: CompanionCommonCallbackContext): Promise<string> {
    return await context.parseVariablesInString("$(lvvmix:foc_player_id)");
}

async function initPlayerOption<T extends  InstanceBaseExt<Config>>(action: CompanionActionEvent, instance: T): Promise<Player> {
    if (typeof action.options.focused_player === "string") {
        return await instance.coordinator.setFocusedPlayer(action.options.focused_player);
    } else {
        return await instance.coordinator.focusedPlayer();
    }
}

async function exitPlayerOption<T extends InstanceBaseExt<Config>>(instance: T,context: CompanionCommonCallbackContext, currentId: string) {
    const previousId = await parseAuto(context);
    if (currentId !== previousId) {
        await instance.coordinator.setFocusedPlayer(previousId)
    }
}

export const setActionDefinitions = <T extends InstanceBaseExt<Config>>(instance: T): CompanionActionDefinitions => {
    const actions: CompanionActionDefinitions = {};
    actions[ActionId.LeaderboardUpdate] = {
        name: 'Leaderboard update',
        options: [],
        callback: async () => {
            await instance.coordinator.updateLeaderboard();
        },
    }
    actions[ActionId.IncreaseScore] = {
        name: 'Increase score',
        options: [
            {
                type: 'dropdown',
                label: 'Choose an option',
                id: 'focused_player',
                default: 'none', // Set the default value to 'none'
                choices: instance.focused_players,
            },
        ],
        callback: async (action, context) => {
            const focusedPlayer = await initPlayerOption(action,instance);

            if (focusedPlayer.holes_finished <= await instance.coordinator.currentHole()) {
                await instance.coordinator.increaseScore();
            }

            await exitPlayerOption(instance, context, focusedPlayer.id);
        },
    }

    actions[ActionId.RevertScoreIncrease] = {
        name: 'Revert score increase',
        options: [],
        callback: async () => {
            await instance.coordinator.revertScore();
            instance.setVariableValues({hole:await instance.coordinator.currentHole()});
        },
    }


    actions[ActionId.ChangeFocusedPlayer] = {
        name: 'Change focused player',
        options: [
            {
                type: 'dropdown',
                label: 'Choose an option',
                id: 'focused_player',
                default: 'none', 
                choices: instance.focused_players,
            },
        ],
        callback: async (action) => {
            const focusedPlayer = await initPlayerOption(action, instance);
            instance.setVariableValues({
                player_name: focusedPlayer.name,
                hole: focusedPlayer.holes_finished,
                foc_player_id: focusedPlayer.id,
            })
        },
    }

    actions[ActionId.IncreaseThrow] = {
        name: 'Increase throw',
        options: [
            {
                type: 'dropdown',
                label: 'Choose an option',
                id: 'focused_player',
                default: 'none', 
                choices: instance.focused_players,
            },
        ],
        callback: async (action, context) => {
            const focusedPlayer = await initPlayerOption(action, instance);
            await instance.coordinator.increaseThrow();
            await exitPlayerOption(instance, context, focusedPlayer.id);
        },
    }
    actions[ActionId.DecreaseThrow] = {
        name: 'Decrease throw',
        options: [
            {
                type: 'dropdown',
                label: 'Choose an option',
                id: 'focused_player',
                default: 'none',
                choices: instance.focused_players,
            },
        ],
        callback: async (action, context) => {
            const focusedPlayer = await initPlayerOption(action, instance);
            await instance.coordinator.revertThrow();
            await exitPlayerOption(instance, context, focusedPlayer.id);
        },
    }
    actions[ActionId.OB] = {
        name: 'OB',
        options: [],
        callback: async () => {
            await instance.coordinator.playObAnimation();
        },
    }
    actions[ActionId.RunAnimation] = {
        name: 'Run animation',
        options: [
            {
                type: 'dropdown',
                label: 'Focused player',
                id: 'focused_player',
                default: 'none', 
                choices: instance.focused_players,
            },
        ],
        callback: async (action, context) => {
            const focusedPlayer = await initPlayerOption(action, instance);

            await instance.coordinator.playAnmiation();
            await exitPlayerOption(instance, context, focusedPlayer.id);
        },
    }

    actions[ActionId.SetHoleInfo] = {
        name: 'Set hole info',
        options: [],
        callback: async () => {
            await instance.coordinator.setHoleInfo();
        }
    }
    actions[ActionId.DoOtherLeaderboard] = {
        name: 'Do other leaderboard',
        options: [
            {
                type: 'number',
                label: 'division number',
                id: 'division',
                default: 1,
                min: 1,
                max: 100
            },
        ],
        callback: async (action) => {
            let div = action.options.division
            if (typeof div === "string" ) {
                await instance.coordinator.doOtherLeaderboard(div);
            }
        }
    }

    return actions
}