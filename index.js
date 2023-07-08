const { InstanceBase, Regex, TCPHelper, InstanceStatus, runEntrypoint, combineRgb} = require('@companion-module/base')
const wasm = require('./rust_pkg')
const net = require('net')
const upgradeScripts = require('./upgrades')



class ModuleInstance extends InstanceBase {
	constructor(internal) {
		super(internal)
	}

	async init(config) {
		console.log('HIII')
		this.updateStatus(InstanceStatus.Ok)
		this.rust_main = new wasm.MyApp()
		console.log('Rust module initialized')
		this.config = config

		this.setVariableDefinitions([
			{
				name: 'Focused player name',
				variableId: 'player_name',
				default: 'z',
			},
		])
		this.initActions()
		this.initFeedbacks()
		if (typeof this.players === 'undefined') {
			this.players = []
		}
		this.players.push({
			id: 'none',
			label: 'None',
		})
		if (typeof this.div_names === 'undefined') {
			this.div_names = []
		}
		this.config.p1 = 'none'
		this.config.p2 = 'none'
		this.config.p3 = 'none'
		this.config.p4 = 'none'
		this.config.div = 'none'
		this.config.round = 1
		this.saveConfig(config)
		
		this.div_names.push({
			id: 'none',
			label: 'None',
		})
		this.foc_player_ind = 0
		this.setVariableValues({
			player_name: '',
			p1: '',
			p2: '',
			p3: '',
			p4: '',
		})

		this.hole = 0
		if (typeof this.focused_players === 'undefined') {
			this.focused_players = []
		}
		this.focused_players = [
			{
				id: 'none',
				label: 'None',
			},
		]
	}


	getConfigFields() {
		return [
			{
				type: 'static-text',
				id: 'info',
				width: 12,
				label: 'Information',
				value: 'Configure your device connection and settings.',
			},
			{
				type: 'textinput',
				id: 'vmix_ip',
				label: 'vMix IP Adress',
				width: 6,
				regex: this.REGEX_IP,
				default: '192.168.120.135',
			},
			{
				type: 'textinput',
				id: 'event_id',
				label: 'Event ID',
				width: 6,
				default: 'a57b4ed6-f64a-4710-8f20-f93e82d4fe79',
				required: true,
			},
			{
				type: 'textinput',
				id: 'vmix_input_id',
				label: 'vMix input ID',
				width: 6,
				default: '1e8955e9-0925-4b54-9e05-69c1b3bbe5ae',
				required: true,
			},
			{
				type: 'number',
				id: 'pool_ind',
				label: 'Pool index',
				width: 6,
				min: 1,
				max: 1000,
				required: true,
				default: 1,
			},
			{
				type: 'number',
				id: 'round',
				label: 'Round',
				width: 6,
				min: 1,
				max: 10,
				default: 1,
			},
			{
				type: 'dropdown',
				id: 'div',
				label: 'Division',
				width: 6,
				default: 'none',
				choices: this.div_names,
			},
			{
				type: 'dropdown',
				id: 'p1',
				label: 'Player 1',
				width: 6,
				default: 'none',
				choices: this.players,
			},
			{
				type: 'dropdown',
				id: 'p2',
				label: 'Player 2',
				width: 6,
				default: 'none',
				choices: this.players,
			},
			{
				type: 'dropdown',
				id: 'p3',
				label: 'Player 3',
				width: 6,
				default: 'none',
				choices: this.players,
			},
			{
				type: 'dropdown',
				id: 'p4',
				label: 'Player 4',
				width: 6,
				default: 'none',
				choices: this.players,
			},
		]
	}

	

	destroy() {
		this.debug('destroy', this.id)
	}

	initFeedbacks() {
		const feedbacks = {
			display_variable: {
				type: 'boolean',
				name: 'Display variable',
				description: 'Displays the exposed variable on the button',
				defaultStyle: {
					color: combineRgb(255, 255, 255),
					bgcolor: combineRgb(0, 0, 255),
				},
				options: [
					{
						type: 'dropdown',
						label: 'Choose an option',
						id: 'chosen_player',
						default: 'none',
						choices: this.focused_players,
					},
				],
				callback: (feedback) => {
					const chosen_player = feedback.options.chosen_player
					console.log(chosen_player)
					console.log(this.foc_player_ind)
					return chosen_player == this.foc_player_ind
				},
			},
		}
		this.setFeedbackDefinitions(feedbacks)
	}
	initActions() {
		this.setActionDefinitions({
			increase_score: {
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
						this.rust_main.set_foc(foc_player)
					}
					let inc = this.rust_main.increase_score()
					if (foc_player != 'none') {
						this.rust_main.set_foc(this.foc_player_ind)
					}
					console.log(inc)
					TCPHelper.send(inc)
				},
			},
			revert_score_increase: {
				name: 'Revert score increase',
				options: [],
				callback: () => {
					let inc = this.rust_main.revert_score()
					TCPHelper.send(inc)
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
						})
						this.checkFeedbacks()
					}
				},
			},
			reset_score: {
				name: 'Reset score',
				options: [],
				callback: () => {
					TCPHelper.send(this.rust_main.reset_score())
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
					TCPHelper.send(inc)
					if (foc_player != 'none') {
						this.rust_main.set_foc(this.foc_player_ind)
					}
					console.log(inc)
					TCPHelper.send(inc)
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
					TCPHelper.send(inc)
					if (foc_player != 'none') {
						this.rust_main.set_foc(this.foc_player_ind)
					}
					console.log(inc)
					console.log()
					TCPHelper.send(inc)
				},
			},
			ob: {
				name: 'OB',
				options: [],
				callback: () => {
					TCPHelper.send(this.rust_main.ob_anim())
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
					console.log(thing)
					TCPHelper.send(thing)
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
						TCPHelper.send(this.rust_main.set_round(this.config.round + 1))
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
						TCPHelper.send(this.rust_main.set_round(this.config.round - 1))
						this.saveConfig()
						this.checkFeedbacks('decrement_round')
					}
				},
			},
		})

	}

	sendCommand(cmd) {
		if (cmd !== undefined && cmd != '') {
			console.log('sending command', cmd)
		}
	}

	async configUpdated(config) {
		this.log('debug', 'Config updated')
		console.log(config.vmix_ip, this.config.vmix_ip)

		console.log(config)
		this.config = config
		if (this.config.vmix_input_id) {
			console.log('setting id')
			this.rust_main.id = this.config.vmix_input_id
		}
		if (this.config.vmix_ip) {
			console.log('setting ip')
			this.rust_main.ip = this.config.vmix_ip
		}
		if (this.config.event_id) {
			console.log('setting event id')
			this.rust_main.event_id = this.config.event_id
			this.rust_main
				.get_event()
				.then(() => {
					console.log('here')
					const divs = this.rust_main.get_div_names()
					console.log(divs)
					this.div_names.length = 0
					this.div_names.push({
						id: 'none',
						label: 'None',
					})
					for (const [idx, name] of this.rust_main.get_div_names().entries()) {
						this.div_names.push({
							id: idx,
							label: name,
						})
					}
				})
				.catch((err) => {
					console.log(err)
				})
		}
		console.log(this.div_names)
		if (this.config.pool_ind) {
			this.rust_main.pool_ind = this.config.pool_ind - 1
		}

		if (Number.isInteger(this.config.div)) {
			this.rust_main.div = this.config.div
			this.players.length = 0
			this.players.push({
				id: 'none',
				label: 'None',
			})

			let ids = []
			let names = []
			for (const name of this.rust_main.get_player_names()) {
				names.push(name)
			}
			for (const id of this.rust_main.get_player_ids()) {
				ids.push(id)
			}
			for (const [idx, name] of names.entries()) {
				this.players.push({
					id: ids[idx],
					label: name,
				})
			}
		}
		this.focused_players.length = 0
		this.focused_players.push({
			id: 'none',
			label: 'None',
		})
		let list = [this.config.p1, this.config.p2, this.config.p3, this.config.p4]
		let p_list = []
		for (const [idx, player] of list.entries()) {
			console.log(player)
			if (typeof player === 'string' && player !== 'none') {
				p_list.push(this.rust_main.set_player(idx + 1, player).join('\r\n'))
			}
		}
		console.log(p_list.join('\r\n'))
		TCPHelper.send(p_list)

		for (const [idx, name] of this.rust_main.get_focused_player_names().entries()) {
			this.focused_players.push({
				id: idx,
				label: name,
			})
		}
		this.initActions()
		this.initFeedbacks()

		// Set variable for focused players
		this.rust_main.get_focused_player_names().forEach((name, index) => {
			let name_thing = 'p' + (index + 1)
			let dict = {}
			dict[name_thing] = name
			this.setVariableValues(dict)
		})
		if (this.rust_main.round != this.config.round - 1) {
			TCPHelper.send(this.rust_main.set_round(this.config.round - 1))
			console.log('Round increased')
		}
		console.log('hereeee')
		//console.log(this.rust_main.get_all_rounds())
	}
}

runEntrypoint(ModuleInstance, upgradeScripts)