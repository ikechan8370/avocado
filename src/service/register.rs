use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use log::{debug, info};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use crate::kritor::server::kritor_proto::common::PushMessageBody;
use crate::kritor::server::kritor_proto::{NoticeEvent, RequestEvent};
use crate::kritor::server::{MESSAGE_CHANNEL, NOTICE_CHANNEL, REQUEST_CHANNEL};
use crate::service::status::status;

pub enum Event {
    Notice,
    Message,
    Request,
}

pub type KritorEvent = crate::kritor::server::kritor_proto::event_structure::Event;
pub type EventHandler = dyn Fn(Arc<KritorEvent>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> + Send + Sync;

pub static MESSAGE_HANDLERS: Lazy<Mutex<HashMap<String, Arc<EventHandler>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub static NOTICE_HANDLERS: Lazy<Mutex<HashMap<String, Arc<EventHandler>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub static REQUEST_HANDLERS: Lazy<Mutex<HashMap<String, Arc<EventHandler>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});


pub async fn listen_to_events() {
    let mut message_receiver = {
        let lock = MESSAGE_CHANNEL.lock().await;
        lock.0.clone().subscribe()
    };
    let mut notice_receiver = {
        let lock = NOTICE_CHANNEL.lock().await;
        lock.0.clone().subscribe()
    };
    let mut request_receiver = {
        let lock = REQUEST_CHANNEL.lock().await;
        lock.0.clone().subscribe()
    };
    tokio::spawn(async move {
        while let Ok(event) = message_receiver.recv().await {
            debug!("Received event: {:?}", event);
            let event_arc = Arc::new(KritorEvent::Message(event)); // 将消息体包裹在Arc中

            let handlers = MESSAGE_HANDLERS.lock().await;
            for service in handlers.values() {
                let service_clone = Arc::clone(service); // 克隆Arc，而不是闭包本身
                let event_clone = Arc::clone(&event_arc);
                tokio::spawn(async move {
                    service_clone(event_clone).await;
                });
            }
        }
    });
    tokio::spawn(async move {
        while let Ok(event) = notice_receiver.recv().await {
            debug!("Received event: {:?}", event);
            let event_arc = Arc::new(KritorEvent::Notice(event)); // 将消息体包裹在Arc中

            let handlers = NOTICE_HANDLERS.lock().await;
            for service in handlers.values() {
                let service_clone = Arc::clone(service); // 克隆Arc，而不是闭包本身
                let event_clone = Arc::clone(&event_arc);
                tokio::spawn(async move {
                    service_clone(event_clone).await;
                });
            }
        }
    });
    tokio::spawn(async move {
        while let Ok(event) = request_receiver.recv().await {
            debug!("Received event: {:?}", event);
            let event_arc = Arc::new(KritorEvent::Request(event)); // 将消息体包裹在Arc中

            let handlers = REQUEST_HANDLERS.lock().await;
            for service in handlers.values() {
                let service_clone = Arc::clone(service); // 克隆Arc，而不是闭包本身
                let event_clone = Arc::clone(&event_arc);
                tokio::spawn(async move {
                    service_clone(event_clone).await;
                });
            }
        }
    });
    info!("Listening to events");
    crate::register_handler!("status", status, vec![Event::Message]);
}

#[macro_export]
macro_rules! register_handler {
    ($event:expr, $handler:expr, $event_type:expr) => {
        use std::sync::Mutex;
        use std::sync::Arc;
        use futures::FutureExt;
        use log::{info};
        use crate::service::register::{EventHandler, KritorEvent, Event};
        tokio::spawn(async move {
            info!("Registering handler for event: {}", $event);
            let handler: Box<EventHandler> = Box::new(|event: Arc<KritorEvent>| {
                let future = $handler(event).boxed();
                future
            });
            for et in $event_type {
                match (et) {
                    Event::Notice => {
                        let mut handlers = $crate::service::register::NOTICE_HANDLERS.lock().await;
                        handlers.insert($event.to_string(), Arc::new(handler));
                        break;
                    },
                    Event::Message => {
                        let mut handlers = $crate::service::register::MESSAGE_HANDLERS.lock().await;
                        handlers.insert($event.to_string(), Arc::new(handler));
                        break;
                    },
                    Event::Request => {
                        let mut handlers = $crate::service::register::REQUEST_HANDLERS.lock().await;
                        handlers.insert($event.to_string(), Arc::new(handler));
                        break;
                    },
                }
            }
        });
    };
}