import { CompanionActionDefinitions } from "@companion-module/base";
import { sendCommand } from "./send";
import { Config } from "./config";
import { InstanceBaseExt } from "./util";
import { FeedbackId } from "./feedbacks";
import { CompanionCommonCallbackContext } from "@companion-module/base/dist/module-api/common";





export enum ActionId {
	LeaderboadUpdate = 'leaderboard_update',
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
    SetHoleInfo = 'set_hole_info'
}

async function parseAuto(context: CompanionCommonCallbackContext): Promise<number> {
    return Number.parseInt(await context.parseVariablesInString("$(lvvmix:foc_player_ind)"));
}


export const setActionDefinitions = (instance: InstanceBaseExt<Config>): CompanionActionDefinitions => {
    const actions: CompanionActionDefinitions = {};
    actions[ActionId.LeaderboadUpdate] = {
        name: 'Leaderboard update',
        options: [],
        callback: () => {
            console.log("gonna send lb update")
            sendCommand(instance.rust_main.set_leaderboard().join('\r\n') + '\r\n', instance.config)
            console.log("sent lb update")
            instance.setVariableValues({
                hole: instance.rust_main.hole,
            })
            console.log("set var values")
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
            if (typeof foc_player === "number") {
                instance.rust_main.set_foc(foc_player)
            }
            let inc = instance.rust_main.increase_score()
            let foc_player_ind = await parseAuto(context)
            instance.rust_main.set_foc(foc_player_ind)
            sendCommand(inc.join('\r\n') + '\r\n', instance.config)
            instance.setVariableValues({
                hole: instance.rust_main.hole,
            })
        },
    }

    actions[ActionId.RevertScoreIncrease] = {
        name: 'Revert score increase',
        options: [],
        callback: () => {
            
            let inc = instance.rust_main.revert_score()
            instance.setVariableValues({hole:instance.rust_main.get_hole(true)})
            sendCommand(inc.join('\r\n') + '\r\n', instance.config)
        },
    }

    actions[ActionId.ResetScore] = {
        name: 'Reset score',
        options: [],
        callback: () => {
            sendCommand(instance.rust_main.reset_score().join('\r\n') + '\r\n', instance.config)
            instance.setVariableValues({
                hole: instance.rust_main.hole,
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
                instance.rust_main.set_foc(foc_player)
                // TODO: Impl change throw popup
                instance.setVariableValues({
                    player_name: instance.rust_main.get_foc_p_name(),
                    hole: instance.rust_main.hole,
                    foc_player_ind: foc_player,
                })
                instance.checkFeedbacks(FeedbackId.FocusedPlayer)
            }
        },
    },
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
                instance.rust_main.set_foc(foc_player)
            }
            let inc = [instance.rust_main.increase_throw()]
            sendCommand(inc.join('\r\n') + '\r\n', instance.config)
            let foc_player_ind = await parseAuto(context)
            instance.rust_main.set_foc(foc_player_ind)

            sendCommand(inc.join('\r\n') + '\r\n', instance.config)
        },
    },
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
                instance.rust_main.set_foc(foc_player)
            }
            let inc = [instance.rust_main.decrease_throw()]
            //sendCommand(inc.join('\r\n') + '\r\n', instance.config)
            let foc_player_ind = await parseAuto(context)
            instance.rust_main.set_foc(foc_player_ind)
            sendCommand(inc.join('\r\n') + '\r\n', instance.config)
        },
    }
    actions[ActionId.OB] = {
        name: 'OB',
        options: [],
        callback: () => {
            sendCommand(instance.rust_main.ob_anim().join('\r\n') + '\r\n', instance.config)
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
                instance.rust_main.set_foc(foc_player)
            }
            let thing = instance.rust_main.play_animation()
            sendCommand(thing.join('\r\n') + '\r\n', instance.config)
            let foc_player_ind = await parseAuto(context)
            instance.rust_main.set_foc(foc_player_ind)

        },
    }

    actions[ActionId.IncrementRound] = {
        name: 'Increment Round',
        options: [],
        callback: () => {
            if (instance.config.round !== undefined && instance.config.round < instance.rust_main.rounds) {
                sendCommand(instance.rust_main.set_round(instance.config.round - 1).join('\r\n') + '\r\n', instance.config)
            }
        },
    }
    actions[ActionId.DecrementRound] = {
        name: 'Decrement Round',
        options: [],
        callback: () => {
            if (instance.config.round !== undefined && instance.config.round > 1) {
                sendCommand(instance.rust_main.set_round(instance.config.round - 1).join('\r\n') + '\r\n', instance.config)
            }
        },
    }
    actions[ActionId.ShowAllPos] = {
        name: 'Show all positions',
        options: [],
        callback: () => {
            sendCommand(instance.rust_main.show_all_pos().join('\r\n') + '\r\n', instance.config)
        },
    }
    actions[ActionId.HideAllPos] = {
        name: 'Hide all positions',
        options: [],
        callback: () => {
            sendCommand(instance.rust_main.hide_all_pos().join('\r\n') + '\r\n', instance.config)
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
                instance.rust_main.set_foc(foc_player)
            }
            sendCommand(instance.rust_main.toggle_pos().join('\r\n') + '\r\n', instance.config)
            let foc_player_ind = await parseAuto(context)
            instance.rust_main.set_foc(foc_player_ind)
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
                instance.rust_main.set_foc(foc_player)
            }
            sendCommand(instance.rust_main.hide_pos().join('\r\n') + '\r\n', instance.config)
            let foc_player_ind = await parseAuto(context)
            instance.rust_main.set_foc(foc_player_ind)
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
                instance.rust_main.set_foc(foc_player)
            }
            sendCommand(instance.rust_main.show_pos().join('\r\n') + '\r\n', instance.config)
            let foc_player_ind = await parseAuto(context)
            instance.rust_main.set_foc(foc_player_ind)
        }
    }
    actions[ActionId.SetHoleInfo] = {
        name: 'Set hole info',
        options: [],
        callback: () => {
            let info = instance.rust_main.make_hole_info().join('\r\n') + '\r\n'
            sendCommand(info, instance.config)
        }
    }
    return actions
}