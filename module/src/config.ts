import { Regex, SomeCompanionConfigField } from "@companion-module/base";

export interface WebSocketSubscription {
	url: string;
	variableName: string;
	subpath: string;
	debug_messages?: boolean;

}



export interface Config {
    coordinator_ip: string
}

export const getConfigFields = (): SomeCompanionConfigField[] => {
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
				id: 'coordinator_ip',
				label: 'Coordinator IP Adress',
				width: 3,
				regex: Regex.IP,
				default: '10.170.120.134',
			},

		];
};