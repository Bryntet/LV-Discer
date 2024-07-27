import WebSocket from 'ws';
import { LevandeVideoInstance } from './index';
import {WebSocketSubscription} from "./config";
import {Player} from "./coordinator_communication"; // Make sure to export LevandeVideoInstance



export class WebSocketManager {
    private ws: WebSocket | null = null;
    private instance: LevandeVideoInstance;
    private subscription: WebSocketSubscription;

    constructor(instance: LevandeVideoInstance, config: WebSocketSubscription) {
        this.instance = instance;
        this.subscription = config;
        this.initWebSocket();
    }

    private initWebSocket(): void {
        if (this.ws) {
            this.ws.close();
        }

        this.ws = new WebSocket(this.subscription.url);

        this.ws.on('open', () => {
            this.instance.log('debug', `WebSocket connection opened for ${this.subscription.url}`);
        });

        this.ws.on('message', (data: WebSocket.Data) => {
            this.instance.log('debug', `Message received from ${this.subscription.url}: ${data}`);
            this.updateVariable(data.toString());
        });

        this.ws.on('close', () => {
            this.instance.log('debug', `WebSocket connection closed for ${this.subscription.url}`);
        });

        this.ws.on('error', (error: Error) => {
            this.instance.log('error', `WebSocket error for ${this.subscription.url}: ${error.message}`);
        });
    }

    private updateVariable(value: string): void {
        if (this.subscription.variableName === 'selected_players') {
            JSON.parse(value).forEach((p: any,index:number) => {
                let player = Player.fromJSON(p);
                this.instance.focused_players[index] = player.toDropdown(index);
                this.instance.setVariableValues({[`p${index + 1}`]: player.name});
            });
        }
        this.instance.setVariableValues({ [this.subscription.variableName]: value });
    }

    public reload(): void {
        this.initWebSocket();
    }

    public destroy(): void {
        if (this.ws) {
            this.ws.close();
        }
    }
}