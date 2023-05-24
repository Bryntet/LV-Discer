const deasync = require('deasync');
const instance_skel = require('../../instance_skel')
const wasm = require("./pkg")
const fetch = require('node-fetch')
const net = require('net')


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
		this.client = new net.Socket()
		this.client.setMaxListeners(40)
		this.isConnected = false
		this.second_one_running = false
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
				default: '192.168.120.135',
			},
			{
				type: 'textinput',
				id: 'event_id',
				label: 'Event ID',
				width: 6,
				default: 'a57b4ed6-f64a-4710-8f20-f93e82d4fe79',
			},
			{
				type: 'textinput',
				id: 'vmix_input_id',
				label: 'vMix input ID',
				width: 6,
				default: '1e8955e9-0925-4b54-9e05-69c1b3bbe5ae',
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


	beConnected() {
		if (!this.isConnected) {
			this.client.connect({ port: 8099, host: this.config.vmix_ip }, () => {
				this.isConnected = true
				console.log('Connected to vMix')
			})
		}
	}

	vmixTCP(msg) {
		if (!this.isConnected) {
			console.log("Not connected to vMix, connecting...")
			this.beConnected()
		} 
		console.log(msg.join('\r\n'))
		this.client.write(msg.join("\r\n")+ "\r\n")
		this.client.on('data', (data) => {
			if (data.includes("ERR")) {
				console.log('Received error: \n' + data)
				//this.client.destroy()
			}
		})
		this.client.on('close', () => {
			console.log('Connection closed')
			this.destroy()
			this.isConnected = false
		})

		this.client.on('error', (err) => {
			console.error('There was an error with the connection: ', err)
		})
	}
	

	destroy() {

		this.debug('destroy', this.id)
	}

	init() {
		if (typeof this.players === 'undefined') {
			this.players = []
		}
		this.players.push({ id: 'none', label: 'None' })
		if (typeof this.div_names === 'undefined') {
			this.div_names = []
		}
		this.config.p1 = 'none'
		this.config.p2 = 'none'
		this.config.p3 = 'none'
		this.config.p4 = 'none'
		this.config.div = 'none'
		this.div_names.push({ id: 'none', label: 'None'})
		this.foc_player_ind = 0
		this.setVariable('player_name', "")
		this.setVariable('p1', "")
		this.setVariable('p2', "")
		this.setVariable('p3', "")
		this.setVariable('p4', "")
		
		this.hole = 0
		if (typeof this.focused_players === 'undefined') {
			this.focused_players = []
		}
		this.focused_players = [{ id: 'none', label: 'None'}]
		
	}
	initFeedbacks() {
		const feedbacks = {
			display_variable: {
				type: 'boolean',
				label: 'Display variable',
				description: 'Displays the exposed variable on the button',
				style: {
					color: this.rgb(255, 255, 255),
					bgcolor: this.rgb(0, 0, 255),
				},
				options:  [{
					type: 'dropdown',
					label: 'Choose an option',
					id: 'chosen_player',
					default: 'none',
					choices:this.focused_players
				}],
				callback: (feedback) => {
					const chosen_player = feedback.options.chosen_player
					console.log(chosen_player)
					console.log(this.foc_player_ind)
					return chosen_player == this.foc_player_ind;
					
				}, 
			},

		}
		this.setFeedbackDefinitions(feedbacks)
	}
	initActions() {
		let actions = {
			increase_score: {
				label: 'Increase score',
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
					this.vmixTCP(inc)
				},
			},
			revert_score_increase: {
				label: 'Revert score increase',
				callback: () => {
					let inc = this.rust_main.revert_score()
					this.vmixTCP(inc)
				},
			},
			change_focused_player: {
				label: 'Change focused player',
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
						this.setVariable('player_name', this.rust_main.get_foc_p_name())
						this.checkFeedbacks()
					}
				},
			},
			reset_score: {
				label: 'Reset score',
				callback: () => {
					this.vmixTCP(this.rust_main.reset_score())
				},
			},
			increase_throw: {
				label: 'Increase throw',
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
					this.vmixTCP(inc)
					if (foc_player != 'none') {
						this.rust_main.set_foc(this.foc_player_ind)
					}
					console.log(inc)
					this.vmixTCP(inc)
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
					this.vmixTCP(inc)
					if (foc_player != 'none') {
						this.rust_main.set_foc(this.foc_player_ind)
					}
					console.log(inc)
					console.log()
					this.vmixTCP(inc)
				},
			},
			ob: {
				label: 'OB',
				callback: () => {
					this.vmixTCP(this.rust_main.ob_anim())
				},
			},
			run_animation: {
				label: 'Run animation',
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
					this.vmixTCP(thing)
					if (foc_player != 'none') {
						this.rust_main.set_foc(this.foc_player_ind)
					}
				},
			},
		}
		
		
		this.setActions(actions)
	}

	async runCommands(url_list) {
		for (const url of url_list) {
			try {await fetch(url)} 
			catch (err) {
				for (let i = 0; i < 3; i++) {
					try {
						await fetch(url)
						break
					} catch (err) {
						console.log(err)
					}
				}
			}
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
		this.client.destroy()
		this.isConnected = false
		let resetConnection = false
		console.log(config.vmix_ip, this.config.vmix_ip)
		
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
		if (this.config.event_id) {
			console.log("setting event id")
			this.rust_main.event_id = this.config.event_id
			this.rust_main.get_event().then(() => {
				console.log("here")
				const divs = this.rust_main.get_div_names()
				console.log(divs)
				this.div_names.length = 0
				this.div_names.push({ id: 'none', label: 'None' })
				for (const [idx, name] of this.rust_main.get_div_names().entries()) {
					this.div_names.push({ id: idx, label: name })
				}
			}).catch((err) => {
				console.log(err)
			})
		}
		console.log(this.div_names)


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
		this.focused_players.length = 0
		this.focused_players.push({ id: 'none', label: 'None' })
		let list = [this.config.p1, this.config.p2, this.config.p3, this.config.p4]
		for (const [idx, player] of list.entries()) {
			console.log(player)
			if (typeof player === 'string' && player !== 'none') {
				console.log("setting p1")
				this.vmixTCP(this.rust_main.set_player(idx + 1, player))
			}
		}
		for (const [idx, name] of this.rust_main.get_focused_player_names().entries()) {
			this.focused_players.push({ id: idx, label: name })
		}
		this.initActions()
		this.initFeedbacks()

		// Set variable for focused players
		this.rust_main.get_focused_player_names().forEach((name, index) => {
			this.setVariable("p"+(index+1), name)
		});
		console.log("hereeee")
		console.log(this.rust_main.get_all_rounds())
	}
}

exports = module.exports = instance