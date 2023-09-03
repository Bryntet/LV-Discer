import { CompanionFeedbackDefinitions, DropdownChoice, combineRgb} from "@companion-module/base";

export enum FeedbackId {
    display_variable = 'display_variable',
}

export function setFeedbackDefinitions(focused_players: DropdownChoice[], foc_player_ind: number, ): CompanionFeedbackDefinitions {
    return {
        [FeedbackId.display_variable]: {
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
                    choices: focused_players,
                },
            ],
            callback: (feedback, context) => {
                const chosen_player = feedback.options.chosen_player
                console.log(chosen_player)
                console.log(foc_player_ind)
                console.log(context.parseVariablesInString("foc_player_ind"))
                return chosen_player === foc_player_ind
            },
        }
    }
}