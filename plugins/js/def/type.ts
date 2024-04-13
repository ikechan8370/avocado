// types.ts

export type Event = import("./constants").Event;
export type Logger = import("./constants").Logger;

export type CheckFunction = (reg: RegExp, msg: string) => boolean;
export interface Plugin {
    matches: (event: Event) => boolean,
    process: (event: Event) => Promise<void>
}

