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
		this.players = [
			{id: 'none', label: "None"}
		];
		this.rust_main = new wasm.MyApp
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
		
		this.config.vmix_ip = "37.123.135.170"
		this.config.pool_id = 'a592cf05-095c-439f-b69c-66511b6ce9c6'
		this.config.vmix_input_id = '506fbd14-52fc-495b-8d17-5b924fba64f3'
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
			this.rust_main.get_divs().then((divs) => {
				console.log(this.rust_main.get_div_names())
			})

		}
	}
}

exports = module.exports = instance