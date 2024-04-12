import { CompanionFeedbackDefinitions, combineRgb} from "@companion-module/base";
import { InstanceBaseExt } from "./util";
import { Config } from "./config";

export enum FeedbackId {
    FocusedPlayer = 'display_variable',
}


export function setFeedbackDefinitions(instance: InstanceBaseExt<Config>): CompanionFeedbackDefinitions {
    return {
        [FeedbackId.FocusedPlayer]: {
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
                    choices: instance.focused_players,
                },
            ],
            callback: async (feedback, context) => {
                const chosen_player = feedback.options.chosen_player
                return chosen_player == await context.parseVariablesInString("$(lvvmix:foc_player_ind)")
            },
        }
    }
}