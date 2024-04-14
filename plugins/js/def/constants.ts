
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
    reply?: (msg: [MessageElement] | MessageElement | string, reply?: boolean) => Promise<void>;
    is_master: boolean,
    contact?: Contact,
    bot: AvocadoBot
}
export interface GroupInfo {
    group_id: number,
    group_name: string,
    group_remark: string,
    owner: number,
    admins: number[],
    max_member_count: number,
    member_count: number,
    group_uin: number,
}

export interface GroupMemberInfo {
    uid: string,
    uin: number,
    nick: string,
    age: number,
    unique_title: string,
    unique_title_expire_time: number,
    card: string,
    join_time: number,
    last_active_time: number,
    level: number,
    shut_up_timestamp: number,
    distance?: number,
    honors: number[],
    unfriendly?: boolean,
    card_changeable?: boolean,
}

export interface FriendInfo {
    uid: string,
    uin: number,
    qid: string,
    nick: string,
    remark: string,
    level: number,
    age: number,
    vote_cnt: number,
    gender: number,
    group_id: number,
    ext?: ExtInfo,
}
export interface ExtInfo {
    big_vip?: boolean,
    hollywood_vip?: boolean,
    qq_vip?: boolean,
    super_vip?: boolean,
    voted?: boolean,
}
export interface MessageElement {
    type: 'Text' | 'At' | 'Face' | 'BubbleFace' | 'Reply' | 'Image' | 'Voice' | 'Video' | 'Basketball' | 'Dice' | 'Rps' | 'Poke' | 'Music' | 'Weather' | 'Location' | 'Share' | 'Gift' | 'MarketFace' | 'Forward' | 'Contact' | 'Json' | 'Xml' | 'File' | 'Markdown' | 'Keyboard',
    data: TextElement | AtElement | FaceElement | BubbleFaceElement | ReplyElement | ImageElement | VoiceElement | VideoElement | BasketballElement | DiceElement | RpsElement | PokeElement | MusicElement | WeatherElement | LocationElement | ShareElement | GiftElement | MarketFaceElement | ForwardElement | ContactElement | JsonElement | XmlElement | FileElement | MarkdownElement | KeyboardElement,
}

export interface TextElement {
    text: string,
}

export interface AtElement {
    uid: string,
    uin?: number,
}

export interface FaceElement {
    id: number,
    is_big?: boolean,
    result?: number,
}

export interface BubbleFaceElement {
    id: number,
    count: number,
}

export interface ReplyElement {
    message_id: string,
}

export interface ImageElement {
    file_md5?: string,
    sub_type?: number,
    type?: number,
    data: {
        File?: Uint8Array,
        FileName?: string,
        FilePath?: string,
        FileUrl?: string,
    },
}

export interface VoiceElement {
    file_md5?: string,
    magic?: boolean,
    data: {
        File?: Uint8Array,
        FileName?: string,
        FilePath?: string,
        FileUrl?: string,
    },
}

export interface VideoElement {
    file_md5?: string,
    data: {
        File?: Uint8Array,
        FileName?: string,
        FilePath?: string,
        FileUrl?: string,
    },
}

export interface BasketballElement {
    id: number,
}

export interface DiceElement {
    id: number,
}

export interface RpsElement {
    id: number,
}

export interface PokeElement {
    id: number,
    type: number,
    strength: number,
}

export interface CustomMusicData {
    url: string,
    audio: string,
    title: string,
    author: string,
    pic: string,
}

export interface MusicElement {
    platform: number,
    data: {
        Id?: string,
        Custom?: CustomMusicData,
    },
}

export interface WeatherElement {
    city: string,
    code: string,
}

export interface LocationElement {
    lat: number,
    lon: number,
    title: string,
    address: string,
}

export interface ShareElement {
    url: string,
    title: string,
    content: string,
    image: string,
}

export interface GiftElement {
    qq: number,
    id: number,
}

export interface MarketFaceElement {
    id: string,
}

export interface ForwardElement {
    res_id: string,
    uniseq: string,
    summary: string,
    description: string,
}

export interface ContactElement {
    scene: number,
    peer: string,
}

export interface JsonElement {
    json: string,
}

export interface XmlElement {
    xml: string,
}

export interface FileElement {
    name?: string,
    size?: number,
    expire_time?: number,
    id?: string,
    url?: string,
    biz?: number,
    sub_id?: string,
}

export interface MarkdownElement {
    markdown: string,
}

export interface ButtonActionPermission {
    type: number,
    role_ids: string[],
    user_ids: string[],
}

export interface ButtonAction {
    type: number,
    permission?: ButtonActionPermission,
    unsupported_tips: string,
    data: string,
    reply: boolean,
    enter: boolean,
}

export interface ButtonRender {
    label: string,
    visited_label: string,
    style: number,
}

export interface Button {
    id: string,
    render_data?: ButtonRender,
    action?: ButtonAction,
}

export interface KeyboardRow {
    buttons: Button[],
}

export interface KeyboardElement {
    rows: KeyboardRow[],
    bot_appid: number,
}

export interface Contact {
    scene: 'GROUP' | 'FRIEND' | 'GUILD' | 'STRANGER',
    peer: string,
    sub_peer?: string,
}

export interface AvocadoBot {
    gl: Map<string, GroupInfo>,
    gml: Map<string, Map<string, GroupMemberInfo>>,
    fl: Map<string, FriendInfo>,
    uin: number,
    uid: string,
    nickname?: string,
    sendMessage: (msg: [MessageElement] | MessageElement | string, contact: Contact, reply?: boolean) => Promise<void>,
}