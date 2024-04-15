use crate::service::service::{Elements, KritorContext, Matchable, Service};
use crate::text;
use async_trait::async_trait;
use avocado_common::Event;
use avocado_macro::service;
use log::info;

#[derive(Debug, Clone, Default)]
#[service(name = "repeat", events(Event::Message))]
struct RepeatPlugin;

#[async_trait]
impl Matchable for RepeatPlugin {
    fn matches(&self, context: KritorContext) -> bool {
        if let Some(message) = context.message {
            if let Some(elements) = message.elements.get_text_elements() {
                return elements.iter().any(|ele| ele.text == "!repeat".to_string());
            }
        }
        false
    }
}
#[async_trait]
impl Service for RepeatPlugin {
    async fn process(&self, context: KritorContext) {
        // let text = context.message.unwrap().elements.get_text_elements().unwrap().get(0).unwrap().text.clone();
        // context.set_store("repeat".to_string(), Box::new(text)).await;
        info!("RepeatPlugin");
        context
            .reply_with_quote(vec![text!("please input something")])
            .await
            .unwrap();
        context
            .start_transaction("repeat".to_string(), None)
            .await
            .unwrap()
    }

    async fn transaction(&self, context: KritorContext) {
        info!("RepeatPlugin transaction");
        let trans_name = context.current_transaction_name.read().await;
        if let Some(trans) = trans_name.as_ref() {
            info!("trans: {}", trans);
            if let Some(ref message) = context.message {
                if let Some(elements) = message.elements.get_text_elements() {
                    info!("RepeatPlugin transaction: {}, {:?}", trans, elements);
                    if trans.as_str() == "repeat" {
                        let text = elements.get(0).unwrap().text.clone();
                        context.reply(vec![text!(text)]).await.unwrap();
                        context.stop_transaction().await.unwrap();
                    }
                }
            }
        }
    }
}
