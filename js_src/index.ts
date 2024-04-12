import { InstanceBase, InstanceStatus, runEntrypoint, SomeCompanionConfigField, DropdownChoice, CompanionStaticUpgradeScript, CompanionVariableValues} from '@companion-module/base';
import { Config, getConfigFields } from "./config";
import { setActionDefinitions } from "./actions";
import { setFeedbackDefinitions } from './feedbacks';
import wasm from '../built/rust_pkg/rust_wasm_test_edvin';
import "net";

class LevandeVideoInstance extends InstanceBase<Config> {
	public rust_main = new wasm.FlipUpVMixCoordinator;
	public config: Config = {
		vmix_ip: 'localhost',
		event_id: 'a57b4ed6-f64a-4710-8f20-f93e82d4fe79',
		vmix_input_id: '506fbd14-52fc-495b-8d17-5b924fba64f3',
		round: 1,
		hole: 0,
		div: 'none',
		p1: 'none',
		p2: 'none',
		p3: 'none',
		p4: 'none',
	};
	private players: DropdownChoice[] = [{ id: 'none', label: 'None' }];
	private div_names: DropdownChoice[] = [{ id: "none", label: 'None' }];
	public foc_player_ind: number = 0;
	public foc_player: string = "z";
	public focused_players: DropdownChoice[] = [{ id: 'none', label: 'None' }];
	public hole: number = 0;

	constructor(internal: unknown) {
		super(internal)
	}

	async init(config: Config) {
		console.log('HIII')
		this.updateStatus(InstanceStatus.Ok)
		this.rust_main = new wasm.FlipUpVMixCoordinator()
		console.log('Rust module initialized')
		this.config = config
		
		this.setVariableDefinitions([
			{
				name: 'Focused player name',
				variableId: 'player_name',
			},
			{
				name: 'Player 1 name',
				variableId: 'p1',
			},
			{
				name: 'Player 2 name',
				variableId: 'p2',
			},
			{
				name: 'Player 3 name',
				variableId: 'p3',
			},
			{
				name: 'Player 4 name',
				variableId: 'p4',
			},
			{
				name: "Current hole",
				variableId: "hole",
			},
			{
				name: "Focused player index",
				variableId: "foc_player_ind",
			},
			{
				name: "Round",
				variableId: "round",
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

	varValues(): CompanionVariableValues {
		while (this.focused_players.length < 5) { // First element is always none
			this.focused_players.push({
				id: 'none' + this.focused_players.length,
				label: 'None',
			})
		}
		return {
			player_name: this.foc_player,
			p1: this.focused_players[1].label, 
			p2: this.focused_players[2].label,
			p3: this.focused_players[3].label,
			p4: this.focused_players[4].label,
			hole: this.rust_main.hole,
			foc_player_ind: this.foc_player_ind,
			round: this.rust_main.round + 1,
		}
	}

	
	public getConfigFields(): SomeCompanionConfigField[] {
        return getConfigFields(this.div_names, this.players);
    }

	async destroy() {
		this.log("warn", 'destroy')
	}

	initFeedbacks() {
		this.setFeedbackDefinitions(setFeedbackDefinitions(this))
	}
	intermediaryValuesSet(values: CompanionVariableValues) {
		console.log("here setting values")
		console.log(values)
		if (typeof values.player_name === "string") {
			this.foc_player = values.player_name
		}
		if (typeof values.foc_player_ind === "number") {
			this.foc_player_ind = values.foc_player_ind
		}
		super.setVariableValues(this.varValues())
		console.log("im so god damn cool")
	}

	

	initActions() {
		this.setActionDefinitions(setActionDefinitions(this))
	}

	async configUpdated(config: Config) {
		this.updateVmixIp(config);
		this.log('debug', 'Config updating');
		
		console.log(config);
		this.config = config;
		
		this.updateVmixInputId();
		this.updateEventId();
		this.updateDiv();
		this.updateFocusedPlayers();
		this.updatePList(config);
		const p_list = this.generatePList([this.config.p1, this.config.p2, this.config.p3, this.config.p4]);
		
		this.initActions();
		this.initFeedbacks();
		this.setVariableValues(this.varValues());

		this.setFocusedPlayerVariables();
		this.updateRoundBasedOnConfig(config);
		this.updateHoleIfNecessary(config, p_list);
	}
	updateVmixIp(config: Config) {
		if (config.vmix_ip != this.config.vmix_ip) {
			console.log('setting ip');
			this.rust_main.ip = config.vmix_ip;
			this.config.vmix_ip = config.vmix_ip;
		}
	}

	updateVmixInputId() {
		if (this.config.vmix_input_id) {
			console.log('setting id');
			
		}
	}

	updateEventId() {
		if (this.config.event_id) {
			console.log('setting event id');
			this.rust_main.event_id = this.config.event_id;
			this.rust_main
				.get_event()
				.then(this.handleEvent.bind(this))
				.catch((err) => {
					console.log(err);
				});
		}
	}

	handleEvent() {
		console.log('here');
		
		this.div_names.length = 0;
		this.div_names.push({
			id: 'none',
			label: 'None',
		});
		for (const [idx, name] of this.rust_main.get_div_names().entries()) {
			this.div_names.push({
				id: idx,
				label: name,
			});
		}
	}

	updateDiv() {
		if (typeof this.config.div === "number") {
			this.rust_main.div = this.config.div;
			this.updatePlayers();
		}
	}

	updatePlayers() {
		this.players.length = 0;
		this.players.push({
			id: 'none',
			label: 'None',
		});

		let ids = this.rust_main.get_player_ids();
		let names = this.rust_main.get_player_names();
		for (const [idx, name] of names.entries()) {
			this.players.push({
				id: ids[idx],
				label: name,
			});
		}
	}

	updateFocusedPlayers() {
		this.focused_players.length = 0;
		this.focused_players.push({
			id: 'none',
			label: 'None',
		});

		for (const [idx, name] of this.rust_main.get_focused_player_names().entries()) {
			this.focused_players.push({
				id: idx,
				label: name,
			});
		}
	}

	updatePList(config: Config) {
		let list = [this.config.p1, this.config.p2, this.config.p3, this.config.p4];
		let p_list = this.generatePList(list);

		if (p_list.length != 0) {
			console.log("Sending p_list");
			sendCommand(p_list.join(''), config);
		}
	}

	generatePList(list: any[]): string[] {
		let p_list: string[] = [];
		for (const [idx, player] of list.entries()) {
			if (typeof player === 'string' && player !== 'none') {
				let cmds = this.rust_main.set_player(idx + 1, player);
				for (const cmd of cmds) {
					p_list.push(cmd + '\r\n');
				}
			}
		}
		return p_list;
	}

	

	updateHole(config: Config) {
		if (this.config.hole != 0 && this.generatePList([this.config.p1, this.config.p2, this.config.p3, this.config.p4]).length != 0) {
			setTimeout(() => {
				sendCommand(this.rust_main.set_all_to_hole(this.config.hole).join('\r\n') + '\r\n', config);
			}, 1000);
			this.setVariableValues({
				hole: this.config.hole,
			});
		}
	}
	setFocusedPlayerVariables() {
		this.rust_main.get_focused_player_names().forEach((name, index) => {
			const name_thing = 'p' + (index + 1);
			console.log(name_thing);
			this.setVariableValues({ [name_thing]: name });
		});
	}

	updateRoundBasedOnConfig(config: Config) {
		if (this.rust_main.round != this.config.round - 1) {
			sendCommand(this.rust_main.set_round(this.config.round - 1).join('\r\n') + '\r\n', config);
			console.log('Round increased');
		}
	}

	updateHoleIfNecessary(config: Config, p_list: string[]) {
		console.log('hereeee');
		console.log(this.config.hole);
		console.log(p_list.length);
		
		if (this.config.hole != 0 && p_list.length != 0) {
			console.log('setting hole');
			console.log(this.config.hole);
			setTimeout(() => {
				sendCommand(this.rust_main.set_all_to_hole(this.config.hole).join('\r\n') + '\r\n', config);
			}, 1000);
			this.setVariableValues({
				hole: this.config.hole,
			});
		}
	}

		

	
}
import { example_conversion } from './upgrades'
import { sendCommand } from './send';
const upgradeScripts: CompanionStaticUpgradeScript<Config>[] = [example_conversion]



runEntrypoint(LevandeVideoInstance, upgradeScripts)