use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use log::warn;
use tokio::sync::RwLock;

use crate::{client_err, err};
use crate::bot::bot::Bot;
use crate::kritor::server::kritor_proto::{EventType, NoticeEvent, RequestEvent, SendMessageResponse};
use crate::kritor::server::kritor_proto::common::{AtElement, Contact, Element, FileElement, image_element, ImageElement, PushMessageBody, ReplyElement, Scene, Sender, TextElement, video_element, voice_element};
use crate::kritor::server::kritor_proto::common::element::{Data, ElementType};
use crate::kritor::server::kritor_proto::event_structure::Event;
use crate::kritor::server::kritor_proto::event_structure::Event::{Message};
use crate::kritor::server::kritor_proto::notice_event::Notice;
use crate::model::error::Result;
use crate::service::register::KritorEvent;

#[derive(Debug, Clone)]
pub struct KritorContext {
    pub r#type: EventType,
    pub message: Option<PushMessageBody>,
    pub notice: Option<NoticeEvent>,
    pub request: Option<RequestEvent>,
    pub bot: Arc<RwLock<Bot>>,
    pub current_service_name: Arc<RwLock<Option<String>>>,
    pub current_transaction_name: Arc<RwLock<Option<String>>>,
    pub store: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
    pub is_master: bool,
}

impl KritorContext {
    pub fn new(event: KritorEvent, bot: Arc<RwLock<Bot>>, service_name: String, is_master: bool) -> Self {
        let mut s = Self {
            r#type: EventType::Message,
            message: None,
            notice: None,
            request: None,
            bot,
            current_service_name: Arc::new(RwLock::new(Some(service_name))),
            current_transaction_name: Arc::new(RwLock::new(None)),
            store: Arc::new(Default::default()),
            is_master,
        };
        match event {
            KritorEvent::Message(message) => {
                s.message = Some(message);
                s.r#type = EventType::Message;
            }
            KritorEvent::Request(request) => {
                s.request = Some(request);
                s.r#type = EventType::Request;
            }
            KritorEvent::Notice(notice) => {
                s.notice = Some(notice);
                s.r#type = EventType::Notice;
            }
        };
        s
    }

    pub async fn set_store(&self, key: String, value: Box<dyn Any + Send + Sync>) {
        let mut store = self.store.write().await;
        store.insert(key, value);
    }
    pub async fn reply(&self, elements: Vec<Element>) -> Result<SendMessageResponse> {
        match self.r#type {
            EventType::Message => {
                let bot_guard = self.bot.read().await;
                let msg = self.message.as_ref().cloned().unwrap();
                bot_guard.send_msg(elements, msg.contact.as_ref().cloned().unwrap()).await
            }
            EventType::Notice => {
                let event = self.notice.as_ref().map(|n| Event::Notice(n.clone())).unwrap();
                let contact = get_concat_from_event(&event).0.unwrap();
                let bot_guard = self.bot.read().await;
                bot_guard.send_msg(elements, contact).await
            }
            EventType::Request => {
                client_err!("Cannot reply to request")
            }
            _ => {
                err!("Unknown event type")
            }
        }
    }

    pub async fn reply_with_quote(&self, elements: Vec<Element>) -> Result<SendMessageResponse> {
        let mut elements = elements;
        if self.r#type == EventType::Message {
            elements.insert(0, Element {
                r#type: i32::from(ElementType::Reply),
                data: Some(Data::Reply(ReplyElement {
                    message_id: self.message.as_ref().cloned().unwrap().message_id.clone(),
                })),
            });
        }
        self.reply(elements).await
    }

    pub async fn start_transaction(&self, name: String, until: Option<u64>) -> Result<()> {
        // 通知该Bot接下来的until秒内 收到消息不再进入任何插件，而是进入该service，且保持context不变
        let context = self.clone();
        let mut trans_name = context.current_transaction_name.write().await;
        *trans_name = Some(name);
        let bot = self.bot.clone();
        // 因为只有message才会进这里，所以这里的message一定是有的
        Bot::stop_broadcast(bot.clone(), context.clone(), self.message.as_ref().unwrap().contact.clone().unwrap(), self.message.as_ref().unwrap().sender.clone().unwrap()).await?;

        // 等待duration后恢复
        let self_clone = self.clone();
        tokio::spawn(async move {
            let duration = until.unwrap_or(30);
            tokio::time::sleep(Duration::from_secs(duration)).await;
            self_clone.stop_transaction().await.unwrap();
        });

        Ok(())
    }

    pub async fn stop_transaction(&self) -> Result<()> {
        let bot = self.bot.clone();
        // 让bot继续广播并结束本次事务
        Bot::resume_broadcast(bot, self.message.as_ref().unwrap().contact.clone().unwrap(), self.message.as_ref().unwrap().sender.clone().unwrap()).await
    }
}

#[async_trait]
pub trait Service: Matchable {
    fn pre_process(&self, context: KritorContext) -> KritorContext {
        context
    }

    async fn process(&self, context: KritorContext);

    async fn transaction(&self, _context: KritorContext) {
        warn!("default transaction");
    }
}

#[async_trait]
pub trait Matchable {
    fn matches(&self, _context: KritorContext) -> bool {
        false
    }
}

impl dyn Service + Send + Sync {}

pub trait Elements {
    fn get_text_elements(&self) -> Option<Vec<TextElement>>;

    fn get_image_elements(&self) -> Option<Vec<ImageElement>>;

    fn get_file_element(&self) -> Option<FileElement>;

    fn get_at_elements(&self) -> Option<Vec<AtElement>>;

    fn get_reply_element(&self) -> Option<ReplyElement>;

    fn get_raw_msg(&self) -> String;
}

impl Elements for Vec<Element> {
    fn get_text_elements(&self) -> Option<Vec<TextElement>> {
        let elements: Vec<TextElement> = self.iter().filter(|ele| ele.r#type == i32::from(ElementType::Text)).map(|ele| ele.data.clone().unwrap()).map(|data| {
            if let Data::Text(text_element) = data {
                text_element
            } else {
                panic!("Element is not a text element")
            }
        }).collect();
        (!elements.is_empty()).then_some(elements)
    }

    fn get_image_elements(&self) -> Option<Vec<ImageElement>> {
        let elements: Vec<ImageElement> = self.iter().filter(|ele| ele.r#type == i32::from(ElementType::Image)).map(|ele| ele.data.clone().unwrap()).map(|data| {
            if let Data::Image(image_element) = data {
                image_element
            } else {
                panic!("Element is not an image element")
            }
        }).collect();
        (!elements.is_empty()).then_some(elements)
    }

    fn get_file_element(&self) -> Option<FileElement> {
        let elements: Vec<FileElement> = self.iter().filter(|ele| ele.r#type == i32::from(ElementType::File)).map(|ele| ele.data.clone().unwrap()).map(|data| {
            if let Data::File(file_element) = data {
                file_element
            } else {
                panic!("Element is not a file element")
            }
        }).collect();
        (!elements.is_empty()).then_some(elements.get(0).unwrap().clone())
    }

    fn get_at_elements(&self) -> Option<Vec<AtElement>> {
        let elements: Vec<AtElement> = self.iter().filter(|ele| ele.r#type == i32::from(ElementType::At)).map(|ele| ele.data.clone().unwrap()).map(|data| {
            if let Data::At(at_element) = data {
                at_element
            } else {
                panic!("Element is not an at element")
            }
        }).collect();
        (!elements.is_empty()).then_some(elements)
    }

    fn get_reply_element(&self) -> Option<ReplyElement> {
        let elements: Vec<ReplyElement> = self.iter().filter(|ele| ele.r#type == i32::from(ElementType::Reply)).map(|ele| ele.data.clone().unwrap()).map(|data| {
            if let Data::Reply(reply_element) = data {
                reply_element
            } else {
                panic!("Element is not a reply element")
            }
        }).collect();
        (!elements.is_empty()).then_some(elements.get(0).unwrap().clone())
    }

    fn get_raw_msg(&self) -> String {
        let mut msg = String::new();

        for ele in self {
            let r#type = ElementType::try_from(ele.r#type).unwrap();
            match r#type {
                ElementType::Text => {
                    if let Data::Text(text_element) = ele.data.clone().unwrap() {
                        msg.push_str(&text_element.text);
                    }
                }
                ElementType::At => {
                    if let Data::At(at_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[@{}]", at_element.uid));
                    }
                }
                ElementType::Face => {
                    if let Data::Face(face_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[表情(id={})]", face_element.id));
                    }
                }
                ElementType::BubbleFace => {
                    if let Data::BubbleFace(bubble_face_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[气泡表情(id={})]", bubble_face_element.id));
                    }
                }
                ElementType::Reply => {
                    if let Data::Reply(reply_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[回复({})]", reply_element.message_id));
                    }
                }
                ElementType::Image => {
                    if let Data::Image(image_element) = ele.data.clone().unwrap() {
                        let data = image_element.data.unwrap();
                        let image = match data {
                            image_element::Data::FileUrl(url) => url,
                            _ => String::default()
                        };
                        msg.push_str(&format!("[图片({})]", image));
                    }
                }
                ElementType::Voice => {
                    if let Data::Voice(voice_element) = ele.data.clone().unwrap() {
                        let data = voice_element.data.unwrap();
                        let image = match data {
                            voice_element::Data::FileUrl(url) => url,
                            _ => String::default()
                        };
                        msg.push_str(&format!("[语音({})]", image));
                    }
                }
                ElementType::Video => {
                    if let Data::Video(video_element) = ele.data.clone().unwrap() {
                        let data = video_element.data.unwrap();
                        let image = match data {
                            video_element::Data::FileUrl(url) => url,
                            _ => String::default()
                        };
                        msg.push_str(&format!("[视频({})]", image));
                    }
                }
                ElementType::Basketball => {
                    if let Data::Basketball(basketball_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[篮球({})]", basketball_element.id));
                    }
                }
                ElementType::Dice => {
                    if let Data::Dice(dice_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[骰子({})]", dice_element.id));
                    }
                }
                ElementType::Rps => {
                    if let Data::Rps(rps_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[Rps({})]", rps_element.id));
                    }
                }
                ElementType::Poke => {
                    if let Data::Poke(poke_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[戳一戳({})]", poke_element.id));
                    }
                }
                ElementType::Music => {
                    if let Data::Music(music_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[音乐({:?})]", music_element));
                    }
                }
                ElementType::Weather => {
                    if let Data::Weather(weather_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[天气({:?})]", weather_element));
                    }
                }
                ElementType::Location => {
                    if let Data::Location(_location_element) = ele.data.clone().unwrap() {
                        msg.push_str(&"[位置]".to_string());
                    }
                }
                ElementType::Share => {
                    if let Data::Share(share_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[分享(title={}, url={})]", share_element.title, share_element.url));
                    }
                }
                ElementType::Gift => {
                    if let Data::Gift(gift_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[礼物(qq={}, id={})]", gift_element.qq, gift_element.id));
                    }
                }
                ElementType::MarketFace => {
                    if let Data::MarketFace(market_face_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[MarketFace({})]", market_face_element.id));
                    }
                }
                ElementType::Forward => {
                    if let Data::Forward(forward_element) = ele.data.clone().unwrap() {
                        // todo forward download
                        msg.push_str(&format!("[转发(res_id={}, uniseq={}, summary={}, description={})]", forward_element.res_id, forward_element.uniseq, forward_element.summary, forward_element.description));
                    }
                }
                ElementType::Contact => {
                    if let Data::Contact(contact_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[名片(scene={}, peer={})]", Scene::try_from(contact_element.scene).unwrap_or_default().as_str_name(), contact_element.peer));;
                    }
                }
                ElementType::Json => {
                    if let Data::Json(json_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[Json({})]", json_element.json));
                    }
                }
                ElementType::Xml => {
                    if let Data::Xml(xml_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[Xml({})]", xml_element.xml));
                    }
                }
                ElementType::File => {
                    if let Data::File(file_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[文件(name={}, url={})]", file_element.name.clone().unwrap_or_default(), file_element.url.clone().unwrap_or_default()));
                    }
                }
                ElementType::Markdown => {
                    if let Data::Markdown(markdown_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[Markdown({})]", markdown_element.markdown));
                    }
                }
                ElementType::Keyboard => {
                    if let Data::Keyboard(keyboard_element) = ele.data.clone().unwrap() {
                        msg.push_str(&format!("[按钮({:?})]", keyboard_element));
                    }
                }
            }
        }
        msg
    }
}

pub fn get_concat_from_event(event: &Event) -> (Option<Contact>, Option<Sender>) {
    match event {
        Message(message) => {
            return (Some(message.contact.as_ref().cloned().unwrap()), Some(message.sender.as_ref().cloned().unwrap()));
        }
        Event::Request(_) => (None, None),
        Event::Notice(notice) => {
            match notice.notice.as_ref().unwrap() {
                Notice::FriendPoke(n) => {
                    let contact = Contact {
                        scene: Scene::Friend.into(),
                        peer: n.operator_uid.clone(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.operator_uid.clone(),
                        uin: Some(n.operator_uin),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::FriendRecall(n) => {
                    let contact = Contact {
                        scene: Scene::Friend.into(),
                        peer: n.operator_uid.clone(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.operator_uid.clone(),
                        uin: Some(n.operator_uin),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::FriendFileUploaded(n) => {
                    let contact = Contact {
                        scene: Scene::Friend.into(),
                        peer: n.operator_uid.clone(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.operator_uid.clone(),
                        uin: Some(n.operator_uin),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::GroupPoke(n) => {
                    let contact = Contact {
                        scene: Scene::Group.into(),
                        peer: n.group_id.to_string(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.operator_uid.clone(),
                        uin: Some(n.operator_uin),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::GroupCardChanged(n) => {
                    let contact = Contact {
                        scene: Scene::Group.into(),
                        peer: n.group_id.to_string(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.operator_uid.clone(),
                        uin: Some(n.operator_uin),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::GroupMemberUniqueTitleChanged(n) => {
                    let contact = Contact {
                        scene: Scene::Group.into(),
                        peer: n.group_id.to_string(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: String::default(),
                        uin: Some(n.target),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::GroupEssenceChanged(n) => {
                    let contact = Contact {
                        scene: Scene::Group.into(),
                        peer: n.group_id.to_string(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.operator_uid.clone(),
                        uin: Some(n.operator_uin),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::GroupRecall(n) => {
                    let contact = Contact {
                        scene: Scene::Group.into(),
                        peer: n.group_id.to_string(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.operator_uid.clone(),
                        uin: Some(n.operator_uin),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::GroupMemberIncrease(n) => {
                    let contact = Contact {
                        scene: Scene::Group.into(),
                        peer: n.group_id.to_string(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.operator_uid.clone(),
                        uin: Some(n.operator_uin),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::GroupMemberDecrease(n) => {
                    let contact = Contact {
                        scene: Scene::Group.into(),
                        peer: n.group_id.to_string(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.target_uid.clone().unwrap(),
                        uin: Some(n.target_uin.unwrap()),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::GroupAdminChange(n) => {
                    let contact = Contact {
                        scene: Scene::Group.into(),
                        peer: n.group_id.to_string(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.target_uid.clone(),
                        uin: Some(n.target_uin),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::GroupMemberBan(n) => {
                    let contact = Contact {
                        scene: Scene::Group.into(),
                        peer: n.group_id.to_string(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.target_uid.clone(),
                        uin: Some(n.target_uin),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::GroupSignIn(n) => {
                    let contact = Contact {
                        scene: Scene::Group.into(),
                        peer: n.group_id.to_string(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.target_uid.clone(),
                        uin: Some(n.target_uin),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::GroupWholeBan(n) => {
                    let contact = Contact {
                        scene: Scene::Group.into(),
                        peer: n.group_id.to_string(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.operator_uid.clone(),
                        uin: Some(n.operator_uin),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
                Notice::GroupFileUploaded(n) => {
                    let contact = Contact {
                        scene: Scene::Group.into(),
                        peer: n.group_id.to_string(),
                        sub_peer: None,
                    };
                    let sender = Sender {
                        uid: n.operator_uid.clone(),
                        uin: Some(n.operator_uin),
                        nick: None,
                    };
                    (Some(contact), Some(sender))
                }
            }
        }
    }
}

#[macro_export]
macro_rules! text {
    ($x:expr) => {
        crate::kritor::server::kritor_proto::common::Element {
            r#type: i32::from(crate::kritor::server::kritor_proto::common::element::ElementType::Text),
            data: Some(crate::kritor::server::kritor_proto::common::element::Data::Text(
                crate::kritor::server::kritor_proto::common::TextElement { text: $x.into() })
            )
        }
    };
}

#[macro_export]
macro_rules! image {
    ($x:expr) => {
        crate::kritor::server::kritor_proto::common::Element {
            r#type: i32::from(crate::kritor::server::kritor_proto::common::element::ElementType::Image),
            data: Some(crate::kritor::server::kritor_proto::common::element::Data::Image(
                crate::kritor::server::kritor_proto::common::ImageElement {
                    file_md5: None,
                    sub_type: None,
                    r#type: Some(i32::from(crate::kritor::server::kritor_proto::common::image_element::ImageType::Common)),
                    data: Some(crate::kritor::server::kritor_proto::common::image_element::Data::File($x)),
                })
            )
        }
    };
}
