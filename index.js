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
		this.rust_main.score_card = new wasm.ScoreCard
		this.initActions()
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
		
		this.saveConfig()
		
	}
	
	initActions() {
		let actions = {}

		actions['sample_action'] = {
			label: 'Get info',
			options: [
				{
					type: 'textinput',
					label: 'Get info from tjing',
					id: 'text',
					regex: this.REGEX_SOMETHING,
				},
			],
			callback: (action, bank) => {
				let opt = action.options
				
				let thing = wasm.test()
				thing.set_player(1, "t")
				this.sendCommand(`SET sample_action: ${opt.text}`)
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

	updateConfig(config) {
		let resetConnection = false
		
		
		console.log(config)
		this.config = config
		if (this.config.vmix_input_id) {
			this.rust_main.id = this.config.vmix_input_id
		}
		if (this.config.vmix_ip) {
			this.rust_main.ip = this.config.vmix_ip
		}
		if (this.config.event_id) {
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
				this.runCommands(this.rust_main.score_card.set_player(idx+1, player)).then(() => {
					console.log("done")
				}).catch((err) => {
					console.log(err)
				})
			}
		}		
	}
}

exports = module.exports = instance