const deasync = require('deasync');
const instance_skel = require('../../instance_skel')
const wasm = require("./node_modules/rust-wasm-test-edvin")
const fetch = require('node-fetch')
// @ts-ignore
global.fetch = fetch;
// @ts-ignore
global.Headers = fetch.Headers;
// @ts-ignore
global.Request = fetch.Request;
// @ts-ignore
global.Response = fetch.Response;
global.AbortController = require('abort-controller')

class instance extends instance_skel {
	constructor(system, id, config) {
		super(system, id, config)
		
		this.rust_main = new wasm.MyApp
		this.setVariableDefinitions([
			{
				label: 'Focused player name',
				name: 'player_name',
				default: 'z',
			},
		])
		this.initActions()
		this.initFeedbacks()
		

	}

	config_fields() {
		return [
			{
				type: 'text',
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
			},
			{
				type: 'textinput',
				id: 'pool_id',
				label: 'Pool ID',
				width: 6,
			},
			{
				type: 'textinput',
				id: 'vmix_input_id',
				label: 'vMix input ID',
				width: 6,
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
		];
	}

	

	destroy() {

		this.debug('destroy', this.id)
	}

	init() {
		this.players = [{ id: 'none', label: 'None' }]
		this.div_names = [{ id: 'none', label: 'None'}]
		this.config.vmix_ip = "37.123.135.170"
		this.config.pool_id = 'a592cf05-095c-439f-b69c-66511b6ce9c6'
		this.config.vmix_input_id = '506fbd14-52fc-495b-8d17-5b924fba64f3'
		this.config.p1 = "none"
		this.config.p2 = "none"
		this.config.p3 = "none"
		this.config.p4 = "none"
		this.config.div = "none"
		this.foc_player_ind = 0
		this.setVariable('player_name', "")
		this.hole = 0
		this.saveConfig()
		
	}
	initFeedbacks() {
		const feedbacks = {
			display_variable: {
				type: 'boolean',
				label: 'Display variable',
				description: 'Displays the exposed variable on the button',
				style: {
					text: '$(lvvmix:player_name)',
					color: this.rgb(255, 255, 255),
					bgcolor: this.rgb(0, 0, 0),
				},
				callback: () => true, // Always return true, so the feedback is always active
			},

		}
		this.setFeedbackDefinitions(feedbacks)
	}
	initActions() {
		let actions = {
			increase_score: {
				label: 'Increase score',
				options: [],
				callback: (action, bank) => {
					let inc = this.rust_main.increase_score()
					console.log(inc)
					this.wrapRunCommands(inc)
				},
			},
			revert_score_increase: {
				label: 'Revert score increase',
				callback: () => {
					let inc = this.rust_main.revert_score()
					this.wrapRunCommands(inc)
				},
			},
			change_focused_player_plus: {
				label: 'Change focused player (+)',
				callback: () => {
					if (this.foc_player_ind < 3) {
						this.foc_player_ind += 1
						this.rust_main.set_foc(this.foc_player_ind)
						// TODO: Impl change throw popup
						this.setVariable('player_name', this.rust_main.get_foc_p_name())
					}
				},
			},
			change_focused_player_minus: {
				label: 'Change focused player (-)',
				callback: () => {
					if (this.foc_player_ind > 0) {
						this.foc_player_ind -= 1
						this.rust_main.set_foc(this.foc_player_ind)
						// TODO: Impl change throw popup
						this.setVariable('player_name', this.rust_main.get_foc_p_name())
					}
				},
			},
			reset_score: {
				label: 'Reset score',
				callback: () => {
					this.wrapRunCommands(this.rust_main.reset_score())
				},
			},
			increase_throw: {
				label: 'Increase throw',
				callback: () => {
					// Your code to increase the throw
				},
			},
			decrease_throw: {
				label: 'Decrease throw',
				callback: () => {
					// Your code to decrease the throw
				},
			},
			ob: {
				label: 'OB',
				callback: () => {
					// Your code for OB
				},
			},
		}
		
		
		this.setActions(actions)
	}

	async runCommands(url_list) {
		for (const url of url_list) {
			console.log(await fetch(url))
		}
	}

	sendCommand(cmd) {
		if (cmd !== undefined && cmd != '') {
			console.log('sending command', cmd)
		}
	}
	wrapRunCommands(cmd) {
		this.runCommands(cmd).then(() => {
			console.log('done')
		}).catch((err) => {
			console.log(err)
		})
	}
	updateConfig(config) {
		let resetConnection = false
		
		
		console.log(config)
		this.config = config
		if (this.config.vmix_input_id) {
			console.log("setting id")
			this.rust_main.id = this.config.vmix_input_id
		}
		if (this.config.vmix_ip) {
			console.log("setting ip")
			this.rust_main.ip = this.config.vmix_ip
		}
		if (this.config.pool_id) {
			console.log("setting event id")
			this.rust_main.pool_id = this.config.pool_id
			this.rust_main.get_divs().then(() => {
				this.div_names.length = 0
				this.div_names.push({ id: 'none', label: 'None' })
				for (const [idx, name] of this.rust_main.get_div_names().entries()) {
					this.div_names.push({ id: idx, label: name })
				}
			})
			if (Number.isInteger(this.config.div)) {
				this.rust_main.div = this.config.div
				this.players.length = 0
				this.players.push({ id: 'none', label: 'None' })

				let ids = []
				let names = []
				for (const name of this.rust_main.get_player_names()) {
					names.push(name)
				}
				for (const id of this.rust_main.get_player_ids()) {
					ids.push(id)
				}
				for (const [idx, name] of names.entries()) {
					this.players.push({ id: ids[idx], label: name })
				}
			}
		}
		let list = [this.config.p1, this.config.p2, this.config.p3, this.config.p4]
		for (const [idx, player] of list.entries()) {
			console.log(player)
			if (typeof player === 'string' && player !== 'none') {
				console.log("setting p1")
				this.wrapRunCommands(this.rust_main.set_player(idx + 1, player))
			}
		}		
	}
}

exports = module.exports = instance