import post, {AxiosResponse} from "axios";

import * as console from "node:console";
import {InstanceBase} from "@companion-module/base";
export class ApiClient {
    baseUrl: string;

    constructor(baseUrl: string) {
        this.baseUrl = baseUrl;
    }

    private async get<T>(endpoint: string): Promise<T> {
        console.log(`${this.baseUrl}${endpoint}`);
        const response = await fetch(`${this.baseUrl}${endpoint}`);
        if (!response.status) {
            if (response.status === 424) {
                throw new Error('Coordinator not initialised');
            } else {
                throw new Error('Network response was not ok');
            }
        }
        let data = response.json();
        console.log(data);
        return data as T;

    }

    private async post(endpoint: string, data?: any): Promise<AxiosResponse<any,any>> {
        return post(`${this.baseUrl}${endpoint}`, data);

    }

    async getRound(): Promise<number> {
        return this.get<number>('/round');
    }

    async amountOfRounds(): Promise<number> {
        return this.get<number>('/rounds');
    }

    async playAnimation(): Promise<void> {
        return this.get<void>('/vmix/play/animation');
    }

    async divisions(): Promise<string[]> {
        return this.get<string[]>('/divisions');
    }

    async currentHole(): Promise<number> {
        return this.get<number>('/current-hole')
    }

    async chosenPlayers(instance: InstanceBase<any>): Promise<Player[]> {

        const playerObjects = await fetch(`${this.baseUrl}/players/focused`);
        instance.log("info", "HELLO THIS MY LOG" + playerObjects.body);
        /*if (!Array.isArray(playerObjects)) {
            throw new Error("Invalid JSON: Expected an array of players");
        }

        console.log(playerObjects);
        return playerObjects.map(obj => {
            return Player.fromJSON(obj);
        });*/
        setTimeout( () => {

        },200);
        return [];
    }


    async focusedPlayer(): Promise<Player> {
        return this.get<Player>(`/player/focused`);
    }

    // Note: This took a boolean previously, unsure why
    async updateLeaderboard() {
        await this.post("/vmix/leaderboard/update");
    }

    async setFocusedPlayer(player_id: string): Promise<Player> {

        const response = await this.post(`/player/focused/set/${player_id}`);
        return Player.fromJSON(response.data);
        
    }

    async increaseScore() {
        await this.post("/vmix/player/focused/score/increase");
    }

    // TODO: Add the following into backend
    async revertScore() {
        await this.post("/vmix/player/focused/score/revert");
    }

    async increaseThrow() {
        await this.post("/vmix/player/focused/throw")
    }

    async revertThrow() {
        await this.post("/vmix/player/focused/revert-throw")
    }

    async playAnmiation() {
        await this.post("/vmix/play/animation")
    }

    async playObAnimation() {
        await this.post("/vmix/play/ob-animation")
    }

    async setHoleInfo() {
        await this.post("/vmix/hole-info/set")
    }

    async doOtherLeaderboard(division: string) {
        await this.post(`/vmix/leaderboard/${division}/update`)
    }

}

export class Player {
    id: string;
    name: string;
    image_url: string | null;
    focused: boolean;
    holes_finished: number;

    constructor(id: string, name: string, focused: boolean, image_url: string | null = null, holes_finished: number) {
        this.id = id;
        this.name = name;
        this.focused = focused;
        this.image_url = image_url;
        this.holes_finished = holes_finished
    }

    static fromJSON(jsonString: string): Player {

        console.log(jsonString);
        const jsonObject = JSON.parse(jsonString);
        if (!jsonObject.id || !jsonObject.name || jsonObject.focused || jsonObject.holes_finished === undefined) {
            throw new Error("Invalid JSON: Missing required player properties");
        }
        return new Player(jsonObject.id, jsonObject.name, jsonObject.focused, jsonObject.image_url || null, jsonObject.holes_finished);
    }
}