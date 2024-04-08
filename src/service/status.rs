use std::sync::Arc;
use std::vec;

use futures::FutureExt;

use crate::kritor::api::send_msg;
use crate::kritor::server::kritor_proto::common::{Element, TextElement};
use crate::kritor::server::kritor_proto::common::element::{Data, ElementType};
use crate::service::register::KritorEvent;

pub async fn status(event: Arc<KritorEvent>) {
    if let KritorEvent::Message(ref message) = *event {
        if let Some(msg) = message.elements.iter().find(|msg| msg.r#type == i32::from(ElementType::Text)).map(|ele| ele.data.clone().unwrap()) {
            let Data::Text(text_element) = msg else { todo!() };
            if text_element.text == "#状态".to_string() {
                send_msg(vec![
                    Element {
                        r#type: i32::from(ElementType::Text),
                        data: Some(Data::Text(TextElement {
                            text: "我状态很好".to_string(),
                        })),
                    }
                ], message.contact.clone().unwrap()).await.unwrap();
            }
        }
    }
}

