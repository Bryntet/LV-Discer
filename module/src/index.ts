import { InstanceBase, InstanceStatus, runEntrypoint, SomeCompanionConfigField, DropdownChoice, CompanionStaticUpgradeScript, CompanionVariableValues} from '@companion-module/base';
import { Config, } from "./config";
import { setActionDefinitions } from "./actions";
import { setFeedbackDefinitions } from './feedbacks';
import { ApiClient } from './coordinator_communication';
class LevandeVideoInstance extends InstanceBase<Config> {
	public coordinator = new ApiClient("http://10.169.122.114:8000");
	public config: Config = {
		vmix_ip: '10.170.120.134',
		event_id: 'd8f93dfb-f560-4f6c-b7a8-356164b9e4be',
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
	public foc_player_id: string = "";
	public foc_player: string = "z";
	public focused_players: DropdownChoice[] = [{ id: 'none', label: 'None' }];
	public hole: number = 0;

	constructor(internal: unknown) {
		console.log("hi");
		super(internal)
	}

	async init(config: Config) {
		console.log('HIII')
		this.updateStatus(InstanceStatus.Ok)

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
				name: "Focused player id",
				variableId: "foc_player_id",
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
		//this.saveConfig(config)

		
		this.foc_player_id = 'none'
		this.setVariableValues(await this.varValues())

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

		console.log("gonna start the queue!")
	}

	/*async startWorker()  {
		const worker = new Worker('./worker.js');  // Ensure this path points to the compiled JS file

		worker.on('message', (message: string) => {
			if (message === 'callFunction') {
				this.rust_main.empty_queue();
			}
		});

		worker.postMessage('start');
	}*/


	async varValues(): Promise<CompanionVariableValues> {
		let focusedPlayers = await this.coordinator.chosenPlayers(this);
		this.log("info", "HELLO");
		this.log("info", focusedPlayers.toString())
		return {
			player_name: this.foc_player,
			p1: focusedPlayers[0].name,
			p2: focusedPlayers[1].name,
			p3: focusedPlayers[2].name,
			p4: focusedPlayers[3].name,
			hole: await this.coordinator.currentHole(),
			foc_player_id: this.foc_player_id,
			round: await this.coordinator.getRound(),
		}
	}

	
	public getConfigFields(): SomeCompanionConfigField[] {
        return [];
    }

	async destroy() {
		this.log("warn", 'destroy')
	}

	initFeedbacks() {
		this.setFeedbackDefinitions(setFeedbackDefinitions(this))
	}


	

	initActions() {
		this.setActionDefinitions(setActionDefinitions(this))
	}

	async configUpdated(config: Config) {
		this.log('debug', 'Config updating');
		
		console.log(config);
		this.config = config;
		await this.updateFocusedPlayers();
		this.initActions();
		this.initFeedbacks();
		this.setVariableValues(await this.varValues());

		await this.setFocusedPlayerVariables();
	}












	async updateFocusedPlayers() {
		this.focused_players.length = 0;
		this.focused_players.push({
			id: 'none',
			label: 'None',
		});

		for (const player of (await this.coordinator.chosenPlayers(this))) {
			this.focused_players.push({
				id: player.id,
				label: player.name,
			});
		}
	}




	


	async setFocusedPlayerVariables() {
		(await this.coordinator.chosenPlayers(this)).forEach((player, index) => {
			const name_thing = 'p' + (index + 1);
			this.setVariableValues({ [name_thing]: player.name });
		});
	}




		

	
}
import { example_conversion } from './upgrades'
import * as console from "console";
const upgradeScripts: CompanionStaticUpgradeScript<Config>[] = [example_conversion]



runEntrypoint(LevandeVideoInstance, upgradeScripts)