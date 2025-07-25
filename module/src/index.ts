import {
    CompanionStaticUpgradeScript,
    CompanionVariableValues,
    DropdownChoice,
    InstanceBase,
    InstanceStatus,
    runEntrypoint,
    SomeCompanionConfigField
} from '@companion-module/base';
import {Config, getConfigFields, WebSocketSubscription} from "./config";
import {setActionDefinitions} from "./actions";
import {setFeedbackDefinitions} from './feedbacks';
import {ApiClient} from './coordinator_communication';
import {example_conversion} from './upgrades'
import * as console from "console";
import {WebSocketManager} from "./websocket_manager";


export class LevandeVideoInstance extends InstanceBase<Config> {
    public config: Config = {
        coordinator_ip: '10.170.122.114'
    };
    public coordinator = new ApiClient(`http://${this.config.coordinator_ip}:8000`);
    private webSocketSubscriptions: WebSocketSubscription[] = [{
        url: `ws://${this.config.coordinator_ip}:8000/ws/players/selected/watch`,
        debug_messages: true,
        variableName: 'selected_players',
        subpath: 'players',
    }, {
        url: `ws://${this.config.coordinator_ip}:8000/ws/hole/watch`,
        debug_messages: true,
        variableName: 'hole',
        subpath: 'hole',
    }, {
        url: `ws://${this.config.coordinator_ip}:8000/ws/hole-finished-alert/watch`,
        debug_messages: true,
        variableName: 'hole_finished_alert',
        subpath: ''
    }];
    public websockets: WebSocketManager[] = [];
    private players: DropdownChoice[] = [{id: 'none', label: 'None'}];
    private div_names: DropdownChoice[] = [{id: "none", label: 'None'}];
    public foc_player_ind: number = 0;
    public foc_player: string = "z"
    public focused_players: DropdownChoice[] = [{id: 'none', label: 'None'}, {id: 0, label: "1"}, {
        id: 1,
        label: "2"
    }, {id: 2, label: "3"}, {id: 3, label: "4"}];
    public hole_finished_alert: string = ""


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
            },
            {
                name: "Hole finished alert",
                variableId: "hole_finished_alert"
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

        this.foc_player_ind = 0;
        this.setVariableValues(await this.varValues())


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

    public async varValues(): Promise<CompanionVariableValues> {
        let focusedPlayers = await this.coordinator.chosenPlayers(this);
        this.log("info", "HELLO");
        this.log("info", focusedPlayers.toString())
        this.foc_player = focusedPlayers[this.foc_player_ind].name;
        return {
            player_name: this.foc_player,
            p1: focusedPlayers[0].name,
            p2: focusedPlayers[1].name,
            p3: focusedPlayers[2].name,
            p4: focusedPlayers[3].name,
            hole: await this.coordinator.currentHole(),
            foc_player_ind: this.foc_player_ind,
            round: await this.coordinator.getRound(),
            hole_finished_alert: this.hole_finished_alert,
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
        this.config.coordinator_ip = config.coordinator_ip;
        this.config = config
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
            this.setVariableValues({[name_thing]: player.name});
        });
    }


}

const upgradeScripts: CompanionStaticUpgradeScript<Config>[] = [example_conversion]


runEntrypoint(LevandeVideoInstance, upgradeScripts)