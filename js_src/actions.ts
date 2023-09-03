import { CompanionActionDefinitions, CompanionVariableValues, DropdownChoice } from "@companion-module/base";
import { MyApp } from "../built/rust_pkg/rust_wasm_test_edvin";
import { sendCommand } from "./send";
import { Config } from "./config";





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

type setVariables = (values: CompanionVariableValues) => void
type feedbacksFunc = (...feedbackTypes: string[]) => void


export const setActionDefinitions = (rust_main: MyApp, config: Config, focused_players: DropdownChoice[], setVariableValues: setVariables, checkFeedbacks: feedbacksFunc, ): CompanionActionDefinitions => {
    const actions: CompanionActionDefinitions = {};
    actions[ActionId.LeaderboadUpdate] = {
        name: 'Leaderboard update',
        options: [],
        callback: () => {
            console.log("gonna send lb update")
            sendCommand(rust_main.set_leaderboard().join('\r\n') + '\r\n', config)
            console.log("sent lb update")
            setVariableValues({
                hole: rust_main.hole,
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
                choices: focused_players,
            },
        ],
        callback: (action) => {
            const foc_player = action.options.focused_player
            if (typeof foc_player === "number") {
                rust_main.set_foc(foc_player)
            }
            let inc = rust_main.increase_score()
            if (typeof foc_player === 'number') {
                rust_main.set_foc(foc_player)
            }
            sendCommand(inc.join('\r\n') + '\r\n', config)
            setVariableValues({
                hole: rust_main.hole,
            })
        },
    }

    actions[ActionId.RevertScoreIncrease] = {
        name: 'Revert score increase',
        options: [],
        callback: () => {
            
            let inc = rust_main.revert_score()
            setVariableValues({hole:rust_main.get_hole(true)})
            sendCommand(inc.join('\r\n') + '\r\n', config)
        },
    }

    actions[ActionId.ResetScore] = {
        name: 'Reset score',
        options: [],
        callback: () => {
            sendCommand(rust_main.reset_score().join('\r\n') + '\r\n', config)
            setVariableValues({
                hole: rust_main.hole,
            })
        },
    },
    actions[ActionId.ChangeFocusedPlayer] = {
        name: 'Change focused player',
        options: [
            {
                type: 'dropdown',
                label: 'Choose an option',
                id: 'focused_player',
                default: 'none', 
                choices: focused_players,
            },
        ],
        callback: (action) => {
            const foc_player = action.options.focused_player
            console.log(foc_player)
            if (typeof foc_player === "number") {
                rust_main.set_foc(foc_player)
                // TODO: Impl change throw popup
                setVariableValues({
                    player_name: rust_main.get_foc_p_name(),
                    hole: rust_main.hole,
                    foc_player_ind: foc_player,
                })
                checkFeedbacks()
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
                choices: focused_players,
            },
        ],
        callback: (action) => {
            const foc_player = action.options.focused_player
            if (typeof foc_player === "number") {
                rust_main.set_foc(foc_player)
            }
            let inc = [rust_main.increase_throw()]
            sendCommand(inc.join('\r\n') + '\r\n', config)
            if (typeof foc_player === 'number') {
                rust_main.set_foc(foc_player)
            }
            sendCommand(inc.join('\r\n') + '\r\n', config)
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
                choices: focused_players,
            },
        ],
        callback: (action) => {
            const foc_player = action.options.focused_player
            if (typeof foc_player === 'number') {
                rust_main.set_foc(foc_player)
            }
            let inc = [rust_main.decrease_throw()]
            sendCommand(inc.join('\r\n') + '\r\n', config)
            if (typeof foc_player === 'number') {
                rust_main.set_foc(foc_player)
            }
            sendCommand(inc.join('\r\n') + '\r\n', config)
        },
    }
    actions[ActionId.OB] = {
        name: 'OB',
        options: [],
        callback: () => {
            sendCommand(rust_main.ob_anim().join('\r\n') + '\r\n', config)
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
                choices: focused_players,
            },
        ],
        callback: (action) => {
            const foc_player = action.options.focused_player
            if (typeof foc_player === 'number') {
                rust_main.set_foc(foc_player)
            }
            let thing = rust_main.play_animation()
            sendCommand(thing.join('\r\n') + '\r\n', config)
            if (typeof foc_player === 'number') {
                rust_main.set_foc(foc_player)
            }
        },
    }

    actions[ActionId.IncrementRound] = {
        name: 'Increment Round',
        options: [],
        callback: () => {
            if (config.round !== undefined && config.round < rust_main.rounds) {
                sendCommand(rust_main.set_round(config.round - 1).join('\r\n') + '\r\n', config)
                checkFeedbacks('increment_round')
            }
        },
    }
    actions[ActionId.DecrementRound] = {
        name: 'Decrement Round',
        options: [],
        callback: () => {
            if (config.round !== undefined && config.round > 1) {
                sendCommand(rust_main.set_round(config.round - 1).join('\r\n') + '\r\n', config)
                checkFeedbacks('decrement_round')
            }
        },
    }
    actions[ActionId.ShowAllPos] = {
        name: 'Show all positions',
        options: [],
        callback: () => {
            sendCommand(rust_main.show_all_pos().join('\r\n') + '\r\n', config)
        },
    }
    actions[ActionId.HideAllPos] = {
        name: 'Hide all positions',
        options: [],
        callback: () => {
            sendCommand(rust_main.hide_all_pos().join('\r\n') + '\r\n', config)
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
                choices: focused_players,
            },
        ],
        callback: (action) => {
            const foc_player = action.options.focused_player
            if (typeof foc_player === 'number') {
                rust_main.set_foc(foc_player)
            }
            sendCommand(rust_main.toggle_pos().join('\r\n') + '\r\n', config)
            if (typeof foc_player === 'number') {
                rust_main.set_foc(foc_player)
            }
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
                choices: focused_players,
            },
        ],
        callback: (action) => {
            const foc_player = action.options.focused_player
            if (typeof foc_player === 'number') {
                rust_main.set_foc(foc_player)
            }
            sendCommand(rust_main.hide_pos().join('\r\n') + '\r\n', config)
            if (typeof foc_player === 'number') {
                rust_main.set_foc(foc_player)
            }
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
                choices: focused_players,
            },
        ],
        callback: (action) => {
            const foc_player = action.options.focused_player
            if (typeof foc_player === 'number') {
                rust_main.set_foc(foc_player)
            }
            sendCommand(rust_main.show_pos().join('\r\n') + '\r\n', config)
            if (typeof foc_player === 'number') {
                rust_main.set_foc(foc_player)
            }
        }
    }
    actions[ActionId.SetHoleInfo] = {
        name: 'Set hole info',
        options: [],
        callback: () => {
            let info = rust_main.make_hole_info().join('\r\n') + '\r\n'
            sendCommand(info, config)
        }
    }
    return actions
}