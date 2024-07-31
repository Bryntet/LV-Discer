import * as console from "node:console";
import {DropdownChoice, InstanceBase} from "@companion-module/base";
import fetch from "node-fetch";

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

    private async post(endpoint: string, data?: any): Promise<any> {
        if (data) {
            return await fetch(`${this.baseUrl}${endpoint}`, {method:"POST",
                body: data, headers: {'Content-Type': 'application/json'}});
        } else {
            return await fetch(`${this.baseUrl}${endpoint}`, {method:"POST"});
        }

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
        return this.get<number>('/hole/current')
    }

    async chosenPlayers(instance: InstanceBase<any>): Promise<Player[]> {

        const playerObjects = await fetch(`${this.baseUrl}/players/card`);
        let players: any = await playerObjects.json();
        let array: Player[] = [];
        for (const player of players) {
            array.push(Player.fromJSON(player));
        }
        instance.log("info", array.toString());
        return array;
    }




    async focusedPlayer(): Promise<Player> {
        return this.get<Player>(`/player/focused`);
    }

    // Note: This took a boolean previously, unsure why
    async updateLeaderboard() {
        await this.post("/vmix/update/leaderboard/");
    }

    async setFocusedPlayer(player_id: number): Promise<Player> {

        const response = await this.post(`/player/focused/set/${player_id}`);
        return Player.fromJSON(response.data);
        
    }

    async increaseScore() {
        await this.post("/vmix/player/focused/score/increase");
    }

    async revertScore() {
        await this.post("/vmix/player/focused/score/revert");
    }

    async increaseThrow() {
        await this.post("/vmix/player/focused/throw/increase")
    }

    async revertThrow() {
        await this.post("/vmix/player/focused/throw/decrease")
    }

    async playAnmiation() {
        await this.post("/vmix/player/focused/animation/play")
    }

    async playObAnimation() {
        await this.post("/vmix/player/focused/animation/play/ob")
    }

    async setHoleInfo() {
        await this.post("/vmix/hole-info/set")
    }

    async doOtherLeaderboard(division: string) {
        await this.post(`/vmix/leaderboard/${division}/update`)
    }

    async doNextQueued() {
        await this.post(`/players/queue/next`)
    }
    
    async setGroupFocusedPlayer() {
        await this.post(`/players/focused/set-group`)
    }
    
    async next10Lb() {
        await this.post(`/leaderboard/next-10`)
    }
    
    async resestLb() {
        await this.post(`/leaderboard/reset-pos`)
    }
    
    async rewindLb() {
        await this.post(`/leaderboard/rewind-pos`)
    }

}

export class Player {
    name: string;
    image_url: string | null;
    focused: boolean;
    holes_finished: number;
    index: number;

    constructor(name: string, focused: boolean, image_url: string | null = null, holes_finished: number, index: number) {
        this.name = name;
        this.focused = focused;
        this.image_url = image_url;
        this.holes_finished = holes_finished;
        this.index = index;
    }

    static fromJSON(jsonObject: any): Player {

        return new Player(jsonObject["name"], jsonObject["focused"], jsonObject["image_url"] || null, jsonObject["holes_finished"], jsonObject["index"]);
    }

    public toDropdown(index: number): DropdownChoice {
        return {"id": index,"label":this.name}
    }
}