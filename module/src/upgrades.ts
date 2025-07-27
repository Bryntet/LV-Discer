import {CompanionStaticUpgradeResult} from "@companion-module/base"
import {Config} from "./config.js"

export function example_conversion(context: any, props: any): CompanionStaticUpgradeResult<Config> {
    if (context + props == 2) {
        console.log("hello")
    }
    const result = {
        updatedConfig: null,
        updatedActions: [],
        updatedFeedbacks: [],
    }
    // write your script in here

    return result
}
