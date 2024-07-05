import { CompanionActionDefinitions } from "@companion-module/base";
import { Config } from "./config";
import { InstanceBaseExt } from "./util";
import { FeedbackId } from "./feedbacks";
import { CompanionCommonCallbackContext } from "@companion-module/base/dist/module-api/common";





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
    IncrementRound = 'increment_round',
    DecrementRound = 'decrement_round',
    ShowAllPos = 'show_all_pos',
    HideAllPos = 'hide_all_pos',
    TogglePos = 'toggle_pos',
    HidePos = 'hide_pos',
    ShowPos = 'show_pos',
    SetHoleInfo = 'set_hole_info',
    DoOtherLeaderboard = 'do_other_leaderboard',
}

async function parseAuto(context: CompanionCommonCallbackContext): Promise<string> {
    return await context.parseVariablesInString("$(lvvmix:foc_player_id)");
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
            const foc_player = action.options.focused_player
            if (typeof foc_player === "string") {
                const focusedPlayer = await instance.coordinator.setFocusedPlayer(foc_player);

                if (focusedPlayer.holes_finished <= await instance.coordinator.currentHole()) {
                    await instance.coordinator.increaseScore();
                }

                let foc_player_id = await parseAuto(context);
                if (focusedPlayer.id != foc_player_id) {

                    let player = await instance.coordinator.setFocusedPlayer(foc_player_id);
                    instance.setVariableValues({
                        hole: player.holes_finished,
                    })
                }

            }

        },
    }

    actions[ActionId.RevertScoreIncrease] = {
        name: 'Revert score increase',
        options: [],
        callback: () => {
            instance.coordinator.revert_score();
            instance.setVariableValues({hole:instance.coordinator.get_hole(true)});
        },
    }

    actions[ActionId.ResetScore] = {
        name: 'Reset score',
        options: [],
        callback: () => {
            instance.coordinator.reset_score();
            instance.setVariableValues({
                hole: instance.coordinator.hole,
            })
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
        callback: (action) => {
            const foc_player = action.options.focused_player
            console.log(foc_player)
            if (typeof foc_player === "number") {
                instance.coordinator.set_foc(foc_player)
                // TODO: Impl change throw popup
                instance.setVariableValues({
                    player_name: instance.coordinator.get_foc_p_name(),
                    hole: instance.coordinator.hole,
                    foc_player_ind: foc_player,
                })
                instance.checkFeedbacks(FeedbackId.FocusedPlayer)
            }
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
            const foc_player = action.options.focused_player
            if (typeof foc_player === "number") {
                instance.coordinator.set_foc(foc_player);
            }
            instance.coordinator.increase_throw();
            let foc_player_ind = await parseAuto(context)
            instance.coordinator.set_foc(foc_player_ind);
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
            const foc_player = action.options.focused_player
            if (typeof foc_player === 'number') {
                instance.coordinator.set_foc(foc_player)
            }
            instance.coordinator.decrease_throw();
            //sendCommand(inc.join('\r\n') + '\r\n', instance.config)
            let foc_player_ind = await parseAuto(context)
            instance.coordinator.set_foc(foc_player_ind);
        },
    }
    actions[ActionId.OB] = {
        name: 'OB',
        options: [],
        callback: () => {
            instance.coordinator.ob_anim();
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
            const foc_player = action.options.focused_player
            if (typeof foc_player === 'number') {
                instance.coordinator.set_foc(foc_player)
            }
            instance.log("debug", "Running animation\nfoc play hole: " + instance.coordinator.focused_player_hole + " hole: " + instance.coordinator.hole)
            if (instance.coordinator.focused_player_hole <= instance.coordinator.hole) {
                instance.coordinator.play_animation();
            }
            let foc_player_ind = await parseAuto(context)
            instance.coordinator.set_foc(foc_player_ind)

        },
    }

    actions[ActionId.IncrementRound] = {
        name: 'Increment Round',
        options: [],
        callback: () => {
            if (instance.config.round !== undefined && instance.coordinator.round + 1 < instance.coordinator.rounds) {
                instance.coordinator.set_round(instance.coordinator.round + 1);
                instance.setVariableValues({ round: instance.coordinator.round + 1 })
                instance.config.round = instance.coordinator.round + 1
            }
        }
    }
    
    actions[ActionId.DecrementRound] = {
        name: 'Decrement Round',
        options: [],
        callback: () => {
            if (instance.config.round !== undefined && instance.coordinator.round > 0) {
                instance.coordinator.set_round(instance.coordinator.round - 1);
                instance.setVariableValues({ round: instance.coordinator.round + 1 })
                instance.config.round = instance.coordinator.round + 1
            }
        },
    }
    actions[ActionId.ShowAllPos] = {
        name: 'Show all positions',
        options: [],
        callback: () => {
            instance.coordinator.show_all_pos();
        },
    }
    actions[ActionId.HideAllPos] = {
        name: 'Hide all positions',
        options: [],
        callback: () => {
            instance.coordinator.hide_all_pos();
        },
    }
    actions[ActionId.TogglePos] = {
        name: 'Toggle current position',
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
            const foc_player = action.options.focused_player
            if (typeof foc_player === 'number') {
                instance.coordinator.set_foc(foc_player)
            }
            //instance.rust_main.toggle_pos();
            let foc_player_ind = await parseAuto(context)
            instance.coordinator.set_foc(foc_player_ind)
        },
    }
    actions[ActionId.HidePos] = {
        name: 'Hide position',
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
            const foc_player = action.options.focused_player
            if (typeof foc_player === 'number') {
                instance.coordinator.set_foc(foc_player)
            }
            instance.coordinator.hide_pos();
            let foc_player_ind = await parseAuto(context);
            instance.coordinator.set_foc(foc_player_ind)
        },
    }
    actions[ActionId.ShowPos] = {
        name: 'Show position',
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
            const foc_player = action.options.focused_player
            if (typeof foc_player === 'number') {
                instance.coordinator.set_foc(foc_player)
            }
            instance.coordinator.show_pos();
            let foc_player_ind = await parseAuto(context)
            instance.coordinator.set_foc(foc_player_ind)
        }
    }
    actions[ActionId.SetHoleInfo] = {
        name: 'Set hole info',
        options: [],
        callback: () => {
            instance.coordinator.make_hole_info();
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
        callback: (action) => {
            let div = action.options.division
            if (typeof div === "number" ) {
                instance.coordinator.make_separate_lb(div-1);
            }
        }
    }

    return actions
}