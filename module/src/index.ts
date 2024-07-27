import { InstanceBase, InstanceStatus, runEntrypoint, SomeCompanionConfigField, DropdownChoice, CompanionStaticUpgradeScript, CompanionVariableValues} from '@companion-module/base';
import {Config, WebSocketSubscription, getConfigFields} from "./config";
import { setActionDefinitions } from "./actions";
import { setFeedbackDefinitions } from './feedbacks';
import { ApiClient } from './coordinator_communication';




export class LevandeVideoInstance extends InstanceBase<Config> {
	public config: Config = {
		coordinator_ip: '10.170.120.134',
	};
	public coordinator = new ApiClient(`http://${this.config.coordinator_ip}:8000`);
	private webSocketSubscriptions: WebSocketSubscription[] = [{
		url: `ws://${this.config.coordinator_ip}:8000/ws/players/selected/watch`,
		debug_messages: true,
		variableName: 'selected_players',
		subpath: 'players',
	},{
		url: `ws://${this.config.coordinator_ip}:8000/ws/hole/watch`,
		debug_messages: true,
		variableName: 'current_hole',
		subpath: 'hole',
	}];
	public websockets: WebSocketManager[] = [];
	private players: DropdownChoice[] = [{ id: 'none', label: 'None' }];
	private div_names: DropdownChoice[] = [{ id: "none", label: 'None' }];
	public foc_player_ind: number = 0;
	public foc_player: string = "z"
	public focused_players: DropdownChoice[] = [{ id: 'none', label: 'None' }];
	public hole: number = 0;

	constructor(internal: unknown) {
		console.log("hi");
		super(internal)
	}

	async init(config: Config) {
		console.log('HIII')
		this.updateStatus(InstanceStatus.Ok)
		for (const sub of this.webSocketSubscriptions) {
			this.websockets.push(new WebSocketManager(this, sub));
		}
		console.log('Rust module initialized')
		this.config = config

		await this.refreshInternalFocusedPlayers();

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
		//this.saveConfig(config)

		
		this.foc_player_ind = 0;
		this.setVariableValues(await this.varValues())

		this.hole = 0

		console.log("gonna start the queue!")
	}


	async refreshInternalFocusedPlayers() {
		this.focused_players = []
		this.focused_players.push({
			id: 'none',
			label: 'None',
		})
		for (const player of (await this.coordinator.chosenPlayers(this))) {
			this.focused_players.push({
				id: player.index,
				label: player.name,
			})
		}

	}

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
			foc_player_ind: this.foc_player_ind,
			round: await this.coordinator.getRound(),
		}
	}

	
	public getConfigFields(): SomeCompanionConfigField[] {
		return getConfigFields()
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
				id: player.index,
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
import {WebSocketManager} from "./websocket_manager";
const upgradeScripts: CompanionStaticUpgradeScript<Config>[] = [example_conversion]



runEntrypoint(LevandeVideoInstance, upgradeScripts)