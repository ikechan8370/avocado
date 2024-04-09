use std::sync::Arc;
use std::vec;

use async_trait::async_trait;
use ctor::ctor;

use crate::kritor::server::kritor_proto::common::{Element, Scene};
use crate::kritor::server::kritor_proto::common::element::Data;
use crate::kritor::server::kritor_proto::common::element::ElementType;
use crate::kritor::server::kritor_proto::common::TextElement;
use crate::service::register::{Event, register_service};
use crate::service::service::{Elements, KritorContext, Service};
use crate::text;

#[derive(Debug, Clone, Default)]
pub struct StatusService;

#[async_trait]
impl Service for StatusService {
    fn matches(&self, context: KritorContext) -> bool {
        if let Some(message) = context.message {
            if let Some(elements) = message.elements.get_text_elements() {
                return elements.iter().any(|ele| ele.text == "!状态");
            }
        }
        false
    }

    async fn process(&self, context: KritorContext) {
        let nickname = context.message.as_ref().and_then(|m| m.sender.as_ref().and_then(|s| s.nick.as_ref())).cloned().unwrap_or_default();
        let uin = context.message.as_ref().and_then(|m| m.sender.as_ref().and_then(|s| s.uin.as_ref())).cloned().unwrap_or_default();
        let uid = context.message.as_ref().and_then(|m| m.sender.as_ref().map(|s| s.uid.as_str())).unwrap_or_default();

        let more = match context.message.as_ref().and_then(|m| m.contact.as_ref().map(|c| c.scene())) {
            Some(Scene::Group) => format!("群号: {}", context.message.as_ref().unwrap().contact.as_ref().unwrap().peer),
            Some(Scene::Friend) => format!("私聊对象: {}", context.message.as_ref().unwrap().contact.as_ref().unwrap().peer),
            _ => "".to_string(),
        };
        context.reply_with_quote(vec![text!(format!("nickname: {}\nuin: {}\nuid: {}\n{}", nickname, uin, uid, more))]).await.unwrap();
    }
}

#[ctor]
fn register() {
   register_service(Arc::new(StatusService::default()), vec![Event::Message], "status".to_string());
}
