import { InstanceBase, TCPHelper, InstanceStatus, runEntrypoint, combineRgb, SomeCompanionConfigField} from '@companion-module/base';
import { Config, getConfigFields } from "./config";
import wasm from '../built/rust_pkg/rust_wasm_test_edvin';
import "net";
import upgradeScripts from './upgrades';




class LevandeVideoInstance extends InstanceBase<Config> {
	private rust_main: wasm.MyApp;
	public config: Config = {
		vmix_ip: 'localhost',
		event_id: 'a57b4ed6-f64a-4710-8f20-f93e82d4fe79',
	};

	constructor(internal: unknown) {
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
			{
				name: 'Player 1 name',
				variableId: 'p1',
				default: 'None',
			},
			{
				name: 'Player 2 name',
				variableId: 'p2',
				default: 'None',
			},
			{
				name: 'Player 3 name',
				variableId: 'p3',
				default: 'None',
			},
			{
				name: 'Player 4 name',
				variableId: 'p4',
				default: 'None',
			},
			{
				name: "Current hole",
				variableId: "hole",
				default: 1
			}
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
		this.setVariableValues(this.varValues())

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

	varValues() {
		return {
			player_name: this.foc_player,
			p1: this.config.p1,
			p2: this.config.p2,
			p3: this.config.p3,
			p4: this.config.p4,
			hole: this.rust_main.hole
		}
	}

	
	public getConfigFields(): SomeCompanionConfigField[] {
        return getConfigFields();
    }

	async destroy() {
		if (this.socket) {
			this.socket.destroy()
		} else {
			this.updateStatus(InstanceStatus.Disconnected)
		}
	}

	initFeedbacks() {
		const feedbacks = {
			display_variable: {
				type: 'boolean',
				name: 'Focused player',
				description: 'Shows if current player is focused',
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
		this.setActionDefinitions()
	}

	

	sendCommand(cmd: string) {
		if (this.config.vmix_ip) {
			let socket = new TCPHelper(this.config.vmix_ip, 8099)

			socket.on('error', (err) => {
				console.log(err)
				this.updateStatus(InstanceStatus.ConnectionFailure, err.message)
				this.log('error', 'Network error: ' + err.message)
			})

			socket.on('data', (data) => {
				if (data.toString().includes('VERSION')) {
					socket.send('PING\r\n')
					socket.send(cmd)
					socket.send('QUIT\r\n')
				}
				if (data.toString().includes("QUIT OK Bye")) {
					socket.destroy()
				}
			})
			console.log('Trying to send command')
			
		} else {
			this.updateStatus(InstanceStatus.BadConfig)
		}

		
	}

	async configUpdated(config) {
		if (config.vmix_ip != this.config.vmix_ip) {
			console.log('setting ip')
			this.rust_main.ip = config.vmix_ip
			this.config.vmix_ip = config.vmix_ip
		}

		this.log('debug', 'Config updating')

		console.log(config)
		this.config = config
		if (this.config.vmix_input_id) {
			console.log('setting id')
			this.rust_main.id = this.config.vmix_input_id
		}
		if (this.config.event_id) {
			console.log('setting event id')
			this.rust_main.event_id = this.config.event_id
			this.rust_main
				.get_event()
				.then(() => {
					console.log('here')
					const divs = this.rust_main.get_div_names()
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
				let cmd = this.rust_main.set_player(idx + 1, player, this.config.round - 1)
				for (const c of cmd) {
					p_list.push(c + '\r\n')
				}
			}
		}
		
		console.log(list)
		console.log("Gonna try p_list")
		console.log(p_list.length)
		if (p_list.length != 0) {
			console.log("Sending p_list")
			this.sendCommand(p_list.join(''))
			
		}


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
			this.sendCommand(this.rust_main.set_round(this.config.round - 1).join('\r\n') + '\r\n')
			console.log('Round increased')
		}
		console.log('hereeee')
		console.log(this.config.hole)
		console.log(p_list.length)
		//console.log(this.rust_main.get_all_rounds())

		if (this.config.hole != 0 && p_list.length != 0) {
			console.log('setting hole')
			console.log(this.config.hole)
			setTimeout(() => {this.sendCommand(this.st_main.set_all_to_hole(this.config.hole).join('\r\n') + '\r\n')}, 1000)
			this.setVariableValues({
				hole: this.config.hole,
			})
			
		}

		

	}
}



runEntrypoint(LevandeVideoInstance, upgradeScripts)