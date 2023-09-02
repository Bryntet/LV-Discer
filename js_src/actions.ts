import { CompanionActionDefinitions } from "@companion-module/base";
import { MyApp } from "../built/rust_pkg/rust_wasm_test_edvin";


export enum ActionId {
	LeaderboadUpdate = 'leaderboard_update',
	IncreaseScore = 'increase_score',
	RevertScoreIncrease = 'revert_score_increase',
    RevertScore = 'revert_score',
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


export const setActionDefinitions = (rust_main: MyApp, current_conf: ): CompanionActionDefinitions => {
    const actions: CompanionActionDefinitions = {};
    actions[ActionId.LeaderboadUpdate] = {
        name: 'Leaderboard update',
				options: [],
				callback: () => {
					console.log("gonna send lb update")
					this.sendCommand(this.rust_main.set_leaderboard().join('\r\n') + '\r\n')
					console.log("sent lb update")
					this.setVariableValues({
						hole: this.rust_main.hole,
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
						choices: this.focused_players,
					},
				],

				callback: (action, bank) => {
					const foc_player = action.options.focused_player

					if (foc_player != 'none') {
						rust_main.set_foc(foc_player)
					}
					let inc = this.rust_main.increase_score()
					if (foc_player != 'none') {
						rust_main.set_foc(this.foc_player_ind)
					}
					this.setVariableValues({
						hole: this.rust_main.hole,
					})
					this.sendCommand(inc.join('\r\n') + '\r\n')
				},
    }




    return {

			increase_score: {
				
			},
			revert_score_increase: {
				name: 'Revert score increase',
				options: [],
				callback: () => {
					let inc = this.rust_main.revert_score()
					this.setVariableValues({hole:this.rust_main.get_hole(true)})
					this.sendCommand(inc.join('\r\n') + '\r\n')
				},
			},
			change_focused_player: {
				name: 'Change focused player',
				options: [
					{
						type: 'dropdown',
						label: 'Choose an option',
						id: 'focused_player',
						default: 'none', // Set the default value to 'none'
						choices: this.focused_players,
					},
				],
				callback: (action) => {
					const foc_player = action.options.focused_player
					this.foc_player_ind = foc_player
					console.log(this.focused_players)
					if (foc_player != 'none') {
						this.rust_main.set_foc(foc_player)
						// TODO: Impl change throw popup
						this.setVariableValues({
							player_name: this.rust_main.get_foc_p_name(),
							hole: this.rust_main.hole,
						})
						this.checkFeedbacks()
					}
				},
			},
			reset_score: {
				name: 'Reset score',
				options: [],
				callback: () => {
					this.sendCommand(this.rust_main.reset_score().join('\r\n') + '\r\n')
					this.setVariableValues({
						hole: this.rust_main.hole,
					})
				},
			},
			increase_throw: {
				name: 'Increase throw',
				options: [
					{
						type: 'dropdown',
						label: 'Choose an option',
						id: 'focused_player',
						default: 'none', // Set the default value to 'none'
						choices: this.focused_players,
					},
				],
				callback: (action) => {
					const foc_player = action.options.focused_player
					if (foc_player != 'none') {
						this.rust_main.set_foc(foc_player)
					}
					let inc = [this.rust_main.increase_throw()]
					this.sendCommand(inc.join('\r\n') + '\r\n')
					if (foc_player != 'none') {
						this.rust_main.set_foc(this.foc_player_ind)
					}
					this.sendCommand(inc.join('\r\n') + '\r\n')
				},
			},
			decrease_throw: {
				label: 'Decrease throw',
				options: [
					{
						type: 'dropdown',
						label: 'Choose an option',
						id: 'focused_player',
						default: 'none', // Set the default value to 'none'
						choices: this.focused_players,
					},
				],
				callback: (action) => {
					const foc_player = action.options.focused_player
					if (foc_player != 'none') {
						this.rust_main.set_foc(foc_player)
					}
					let inc = [this.rust_main.decrease_throw()]
					this.sendCommand(inc.join('\r\n') + '\r\n')
					if (foc_player != 'none') {
						this.rust_main.set_foc(this.foc_player_ind)
					}
					this.sendCommand(inc.join('\r\n') + '\r\n')
				},
			},
			ob: {
				name: 'OB',
				options: [],
				callback: () => {
					this.sendCommand(this.rust_main.ob_anim().join('\r\n') + '\r\n')
				},
			},
			run_animation: {
				name: 'Run animation',
				options: [
					{
						type: 'dropdown',
						label: 'Focused player',
						id: 'focused_player',
						default: 'none', // Set the default value to 'none'
						choices: this.focused_players,
					},
				],
				callback: (action) => {
					const foc_player = action.options.focused_player
					if (foc_player != 'none') {
						this.rust_main.set_foc(foc_player)
					}
					let thing = this.rust_main.play_animation()
					this.sendCommand(thing.join('\r\n') + '\r\n')
					if (foc_player != 'none') {
						this.rust_main.set_foc(this.foc_player_ind)
					}
				},
			},
			increment_round: {
				name: 'Increment Round',
				options: [],
				callback: () => {
					if (this.config.round !== undefined && this.config.round < this.rust_main.rounds) {
						this.config.round++
						this.sendCommand(this.rust_main.set_round(this.config.round - 1).join('\r\n') + '\r\n')
						//this.rust_main.reset_thru()
						this.saveConfig()
						this.checkFeedbacks('increment_round')
					}
				},
			},
			decrement_round: {
				name: 'Decrement Round',
				options: [],
				callback: () => {
					if (this.config.round !== undefined && this.config.round > 1) {
						this.config.round--
						this.sendCommand(this.rust_main.set_round(this.config.round - 1).join('\r\n') + '\r\n')
						//this.rust_main.reset_thru()
						this.saveConfig()
						this.checkFeedbacks('decrement_round')
					}
				},
			},
			show_all_pos: {
				name: 'Show all positions',
				options: [],
				callback: () => {
					this.sendCommand(this.rust_main.show_all_pos().join('\r\n') + '\r\n')
				},
			},
			hide_all_pos: {
				name: 'Hide all positions',
				options: [],
				callback: () => {
					this.sendCommand(this.rust_main.hide_all_pos().join('\r\n') + '\r\n')
				},
			},
			toggle_pos: {
				name: 'Toggle current position',
				options: [
					{
						type: 'dropdown',
						label: 'Choose an option',
						id: 'focused_player',
						default: 'none', // Set the default value to 'none'
						choices: this.focused_players,
					},
				],
				callback: (action) => {
					const foc_player = action.options.focused_player
					if (foc_player != 'none') {
						this.rust_main.set_foc(foc_player)
					}
					this.sendCommand(this.rust_main.toggle_pos().join('\r\n') + '\r\n')
					if (foc_player != 'none') {
						this.rust_main.set_foc(this.foc_player_ind)
					}
				},
			},
			hide_pos: {
				name: 'Hide position',
				options: [
					{
						type: 'dropdown',
						label: 'Choose an option',
						id: 'focused_player',
						default: 'none', // Set the default value to 'none'
						choices: this.focused_players,
					},
				],
				callback: (action) => {
					const foc_player = action.options.focused_player
					if (foc_player != 'none') {
						this.rust_main.set_foc(foc_player)
					}
					this.sendCommand(this.rust_main.hide_pos().join('\r\n') + '\r\n')
					if (foc_player != 'none') {
						this.rust_main.set_foc(this.foc_player_ind)
					}
				},
			},
			show_pos: {
				name: 'Show position',
				options: [
					{
						type: 'dropdown',
						label: 'Choose an option',
						id: 'focused_player',
						default: 'none', // Set the default value to 'none'
						choices: this.focused_players,
					},
				],
				callback: (action) => {
					const foc_player = action.options.focused_player
					if (foc_player != 'none') {
						this.rust_main.set_foc(foc_player)
					}
					this.sendCommand(this.rust_main.show_pos().join('\r\n') + '\r\n')
					if (foc_player != 'none') {
						this.rust_main.set_foc(this.foc_player_ind)
					}
				}
			},
			set_hole_info: {
				name: 'Set hole info',
				options: [],
				callback: () => {
					let info = this.rust_main.make_hole_info().join('\r\n') + '\r\n'
					this.sendCommand(info)
				}
			}
		}
}