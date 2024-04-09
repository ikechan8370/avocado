use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{client_err, err};
use crate::bot::bot::Bot;
use crate::kritor::server::kritor_proto::{EventType, NoticeEvent, RequestEvent, SendMessageResponse};
use crate::kritor::server::kritor_proto::common::{AtElement, Contact, Element, FileElement, ImageElement, PushMessageBody, ReplyElement, Scene, TextElement};
use crate::kritor::server::kritor_proto::common::element::{Data, ElementType};
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
}

impl KritorContext {
    pub fn new(event: KritorEvent, bot: Arc<RwLock<Bot>>) -> Self {
        match event {
            KritorEvent::Message(message) => Self {
                r#type: EventType::Message,
                message: Some(message),
                notice: None,
                request: None,
                bot
            },
            KritorEvent::Request(request) => Self {
                r#type: EventType::Message,
                message: None,
                notice: None,
                request: Some(request),
                bot
            },
            KritorEvent::Notice(notice) => Self {
                r#type: EventType::Message,
                message: None,
                notice: Some(notice),
                request: None,
                bot
            },
        }
    }

    pub async fn reply(&self, elements: Vec<Element>) -> Result<SendMessageResponse> {
        match self.r#type {
            EventType::Message => {
                let bot_guard = self.bot.read().await;
                bot_guard.send_msg(elements, self.message.clone().unwrap().contact.unwrap()).await
            }
            EventType::Notice => {
                let contact = match self.notice.as_ref().unwrap().notice.as_ref().cloned().unwrap() {
                    Notice::FriendPoke(n) => {
                        Contact {
                            scene: Scene::Friend.into(),
                            peer: n.operator_uid,
                            sub_peer: None,
                        }
                    },
                    Notice::FriendRecall(n) => {
                        Contact {
                            scene: Scene::Friend.into(),
                            peer: n.operator_uid,
                            sub_peer: None,
                        }
                    },
                    Notice::FriendFileUploaded(n) => {
                        Contact {
                            scene: Scene::Friend.into(),
                            peer: n.operator_uid,
                            sub_peer: None,
                        }
                    },
                    Notice::GroupPoke(n) => {
                        Contact {
                            scene: Scene::Group.into(),
                            peer: n.group_id.to_string(),
                            sub_peer: None,
                        }
                    },
                    Notice::GroupCardChanged(n) => {
                        Contact {
                            scene: Scene::Group.into(),
                            peer: n.group_id.to_string(),
                            sub_peer: None,
                        }
                    },
                    Notice::GroupMemberUniqueTitleChanged(n) => {
                        Contact {
                            scene: Scene::Group.into(),
                            peer: n.group_id.to_string(),
                            sub_peer: None,
                        }
                    },
                    Notice::GroupEssenceChanged(n) => {
                        Contact {
                            scene: Scene::Group.into(),
                            peer: n.group_id.to_string(),
                            sub_peer: None,
                        }
                    },
                    Notice::GroupRecall(n) => {
                        Contact {
                            scene: Scene::Group.into(),
                            peer: n.group_id.to_string(),
                            sub_peer: None,
                        }
                    },
                    Notice::GroupMemberIncrease(n) => {
                        Contact {
                            scene: Scene::Group.into(),
                            peer: n.group_id.to_string(),
                            sub_peer: None,
                        }
                    },
                    Notice::GroupMemberDecrease(n) => {
                        Contact {
                            scene: Scene::Group.into(),
                            peer: n.group_id.to_string(),
                            sub_peer: None,
                        }
                    },
                    Notice::GroupAdminChange(n) => {
                        Contact {
                            scene: Scene::Group.into(),
                            peer: n.group_id.to_string(),
                            sub_peer: None,
                        }
                    },
                    Notice::GroupMemberBan(n) => {
                        Contact {
                            scene: Scene::Group.into(),
                            peer: n.group_id.to_string(),
                            sub_peer: None,
                        }
                    },
                    Notice::GroupSignIn(n) => {
                        Contact {
                            scene: Scene::Group.into(),
                            peer: n.group_id.to_string(),
                            sub_peer: None,
                        }
                    },
                    Notice::GroupWholeBan(n) => {
                        Contact {
                            scene: Scene::Group.into(),
                            peer: n.group_id.to_string(),
                            sub_peer: None,
                        }
                    },
                    Notice::GroupFileUploaded(n) => {
                        Contact {
                            scene: Scene::Group.into(),
                            peer: n.group_id.to_string(),
                            sub_peer: None,
                        }
                    },
                };
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
                    message_id: self.message.clone().unwrap().message_id,
                })),
            });
        }
        self.reply(elements).await
    }
}

#[async_trait]
pub trait Service {

    fn matches(&self, context: KritorContext) -> bool;

    fn pre_process(&self, context: KritorContext) -> KritorContext {
        context
    }

    async fn process(&self, context: KritorContext);

    fn post_process(&self, context: KritorContext) {

    }
}

impl dyn Service + Send + Sync {}

pub trait Elements {
    fn get_text_elements(&self) -> Option<Vec<TextElement>>;

    fn get_image_elements(&self) -> Option<Vec<ImageElement>>;

    fn get_file_element(&self) -> Option<FileElement>;

    fn get_at_elements(&self) -> Option<Vec<AtElement>>;

    fn get_reply_element(&self) -> Option<ReplyElement>;
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
}


#[macro_export]
macro_rules! text {
    ($x:expr) => {
        Element {
            r#type: i32::from(ElementType::Text),
            data: Some(Data::Text(TextElement { text: $x.into() }))
        }
    };
}
