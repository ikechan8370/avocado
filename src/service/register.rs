use std::collections::HashMap;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{Arc};
use std::sync::mpsc::Receiver;
use log::{debug, info};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;
use tokio::sync::{Mutex, MutexGuard, RwLock};
use crate::bot::bot::Bot;
use crate::kritor::server::kritor_proto::common::PushMessageBody;
use crate::kritor::server::kritor_proto::{NoticeEvent, RequestEvent};
use crate::service::service::{KritorContext, Service};

pub enum Event {
    Notice,
    Message,
    Request,
}

pub type KritorEvent = crate::kritor::server::kritor_proto::event_structure::Event;
pub type EventHandler = Arc<dyn Service + Send + Sync>;

pub static MESSAGE_SERVICES: Lazy<Mutex<HashMap<String, EventHandler>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub static NOTICE_SERVICES: Lazy<Mutex<HashMap<String, EventHandler>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub static REQUEST_SERVICES: Lazy<Mutex<HashMap<String, EventHandler>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create a Tokio runtime")
});

pub static INITIALIZED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));


pub async fn listen_to_events(bot: Arc<RwLock<Bot>>) {
    let bot_guard = bot.read().await;
    let mut message_receiver = bot_guard.subscribe_message();
    let mut notice_receiver = bot_guard.subscribe_notice();
    let mut request_receiver = bot_guard.subscribe_request();
    async fn subscribe(handlers: MutexGuard<'_, HashMap<String, EventHandler>>, event_arc: Arc<KritorEvent>, bot: Arc<RwLock<Bot>>) {
        for service in handlers.values() {
            let service_clone = Arc::clone(service);
            let event_clone = Arc::clone(&event_arc);
            let context = KritorContext::new(event_clone.as_ref().clone(), bot.clone());
            if service_clone.matches(context.clone()) {
                tokio::spawn(async move {
                    service_clone.process(context).await;
                });
            }
        }
    }
    let bot_clone = bot.clone();
    tokio::spawn(async move {
        while let Ok(event) = message_receiver.recv().await {
            debug!("Received event: {:?}", event);
            let event_arc = Arc::new(KritorEvent::Message(event)); // 将消息体包裹在Arc中
            // 使用全局的service处理器，每个bot的消息都会推送到同样的service中
            let handlers = MESSAGE_SERVICES.lock().await;
            subscribe(handlers, event_arc, bot_clone.clone()).await;
        }
    });
    let bot_clone = bot.clone();
    tokio::spawn(async move {
        while let Ok(event) = notice_receiver.recv().await {
            debug!("Received event: {:?}", event);
            let event_arc = Arc::new(KritorEvent::Notice(event)); // 将消息体包裹在Arc中
            let handlers = NOTICE_SERVICES.lock().await;
            subscribe(handlers, event_arc, bot_clone.clone()).await;
        }
    });
    let bot_clone = bot.clone();
    tokio::spawn(async move {
        while let Ok(event) = request_receiver.recv().await {
            debug!("Received event: {:?}", event);
            let event_arc = Arc::new(KritorEvent::Request(event)); // 将消息体包裹在Arc中

            let handlers = REQUEST_SERVICES.lock().await;
            subscribe(handlers, event_arc, bot_clone.clone()).await;
        }
    });
}

pub fn register_service(service: Arc<dyn Service + Send + Sync>, event: Vec<Event>, name: String) {
    let _guard = RUNTIME.enter();
    let future = async {
        _register_service(service, event, name).await;
        let mut initialized = INITIALIZED.lock().await;
        *initialized = true;
    };
    RUNTIME.spawn(future);
}

async fn _register_service(service: Arc<dyn Service + Send + Sync>, event: Vec<Event>, name: String) {
    info!("Registering service: {}", name);
    for et in event {
        match et {
            Event::Notice => {
                let mut handlers = NOTICE_SERVICES.lock().await;
                handlers.insert(name.clone(), service.clone());
            },
            Event::Message => {
                let mut handlers = MESSAGE_SERVICES.lock().await;
                handlers.insert(name.clone(), service.clone());
            },
            Event::Request => {
                let mut handlers = REQUEST_SERVICES.lock().await;
                handlers.insert(name.clone(), service.clone());
            },
        }
    }
}