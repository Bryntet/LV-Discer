import { DropdownChoice, Regex, SomeCompanionConfigField } from "@companion-module/base";

export interface WebSocketSubscription {
	url: string;
	variableName: string;
	subpath: string;
	debug_messages?: boolean;

}



export interface Config {
    vmix_ip: string;
    event_id: string;
    vmix_input_id: string;
    round: number;
    hole: number;
    div: number|string;
    p1: string;
    p2: string;
    p3: string;
    p4: string;
}

export const getConfigFields = (div_names: DropdownChoice[], players: DropdownChoice[]): SomeCompanionConfigField[] => {
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
				width: 3,
				regex: Regex.IP,
				default: '10.170.120.134',
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
				default: '506fbd14-52fc-495b-8d17-5b924fba64f3',
				required: true,
			},
			{
				type: 'number',
				id: 'round',
				label: 'Round',
				width: 2,
				min: 1,
				max: 10,
				default: 1,
			},
			{
				type: 'number',
				id: 'hole',
				label: 'Hole',
				width: 2,
				min: 0,
				max: 18,
				default: 0,
			},
			{
				type: 'dropdown',
				id: 'div',
				label: 'Division',
				width: 12,
				default: 'none',
				choices: div_names,
			},
			{
				type: 'dropdown',
				id: 'p1',
				label: 'Player 1',
				width: 6,
				default: 'none',
				choices: players,
			},
			{
				type: 'dropdown',
				id: 'p2',
				label: 'Player 2',
				width: 6,
				default: 'none',
				choices: players,
			},
			{
				type: 'dropdown',
				id: 'p3',
				label: 'Player 3',
				width: 6,
				default: 'none',
				choices: players,
			},
			{
				type: 'dropdown',
				id: 'p4',
				label: 'Player 4',
				width: 6,
				default: 'none',
				choices: players,
			},
			
		];
};