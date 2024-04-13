
export interface Logger {
    debug: (message: string|any) => void,
    info: (message: string|any) => void,
    error: (message: string|any) => void,
    warn: (message: string|any) => void,
}

export interface Sender {
    uid: string,
    uin?: string | number,
    nick?: string
}

export interface Event {
    type: 'Message' | 'Notice' | 'Request',
    msg?: string;
    sender?: Sender;
    reply?: (msg: string) => Promise<void>;
}
export const e: Event = {
    type: "Message"
};

export interface GroupInfo {

}

export interface GroupMemberInfo {

}

export interface FriendInfo {

}

export interface MessageElement {

}

export interface Contact {

}
export interface AvocadoBot {
    gl: Map<string, GroupInfo>,
    gml: Map<string, Map<string, GroupMemberInfo>>,
    fl: Map<string, FriendInfo>,
    sendMessage: (msg: [MessageElement], contact: Contact, reply?: boolean) => Promise<void>,
}

export const Bot: AvocadoBot = new class implements AvocadoBot {
    constructor() {
        this.fl = new Map();
        this.gl = new Map();
        this.gml = new Map();
    }
    fl: Map<string, FriendInfo>;
    gl: Map<string, GroupInfo>;
    gml: Map<string, Map<string, GroupMemberInfo>>;

    sendMessage(msg: [MessageElement], contact: Contact, reply: boolean | undefined): Promise<void> {
        return Promise.resolve(undefined);
    }
}