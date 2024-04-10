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
    pub current_service_name: Arc<RwLock<Option<String>>>,
    pub current_transaction_name: Arc<RwLock<Option<String>>>,
    pub store: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>
}

impl KritorContext {
    pub fn new(event: KritorEvent, bot: Arc<RwLock<Bot>>, service_name: String) -> Self {
        let mut s = Self {
            r#type: EventType::Message,
            message: None,
            notice: None,
            request: None,
            bot,
            current_service_name: Arc::new(RwLock::new(Some(service_name))),
            current_transaction_name: Arc::new(RwLock::new(None)),
            store: Arc::new(Default::default()),
        };
        match event {
            KritorEvent::Message(message) => {
                s.message = Some(message);
                s.r#type = EventType::Message;
            },
            KritorEvent::Request(request) => {
                s.request = Some(request);
                s.r#type = EventType::Request;
            },
            KritorEvent::Notice(notice) => {
                s.notice = Some(notice);
                s.r#type = EventType::Notice;
            },
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

    pub async fn start_transaction(&self, name: String, until: Option<u64>) -> Result<()> {
        // 通知该Bot接下来的until秒内 收到消息不再进入任何插件，而是进入该service，且保持context不变
        let mut context = self.clone();
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

    async fn transaction(&self, context: KritorContext) {
        warn!("default transaction");
    }
}

#[async_trait]
pub trait Matchable {
    fn matches(&self, context: KritorContext) -> bool {
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
        crate::kritor::server::kritor_proto::common::Element {
            r#type: i32::from(crate::kritor::server::kritor_proto::common::element::ElementType::Text),
            data: Some(crate::kritor::server::kritor_proto::common::element::Data::Text(
                crate::kritor::server::kritor_proto::common::TextElement { text: $x.into() })
            )
        }
    };
}
