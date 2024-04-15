use crate::bot::bot::Bot;
use crate::kritor::server::kritor_proto::event_structure::Event;
use crate::service::register::listen_to_events;
use dashmap::DashMap;
use kritor_proto::event_service_server::EventService;
use kritor_proto::reverse_service_server::ReverseService;
use kritor_proto::{common, EventStructure, RequestPushEvent};
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;
use std::error::Error;
use std::io::ErrorKind;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, Notify, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

pub mod kritor_proto {
    tonic::include_proto!("kritor.event");
    tonic::include_proto!("kritor.authentication");
    tonic::include_proto!("kritor.core");
    tonic::include_proto!("kritor.developer");
    tonic::include_proto!("kritor.file");
    tonic::include_proto!("kritor.friend");
    tonic::include_proto!("kritor.group");
    tonic::include_proto!("kritor.guild");
    tonic::include_proto!("kritor.message");
    tonic::include_proto!("kritor.process");
    tonic::include_proto!("kritor.web");
    tonic::include_proto!("kritor.reverse");
    // tonic::include_proto!("kritor.common");
    pub mod common {
        tonic::include_proto!("kritor.common");
    }
}

static NOTIFY: Notify = Notify::const_new();

pub static BOTS: Lazy<Arc<RwLock<DashMap<String, Arc<RwLock<Bot>>>>>> =
    Lazy::new(|| Arc::new(RwLock::new(DashMap::new())));

#[derive(Debug, Default)]
pub struct EventListener {}

type ResponseStream = Pin<Box<dyn Stream<Item = Result<EventStructure, Status>> + Send>>;

#[tonic::async_trait]
impl EventService for EventListener {
    type RegisterActiveListenerStream = ResponseStream;

    async fn register_active_listener(
        &self,
        _request: Request<RequestPushEvent>,
    ) -> Result<Response<Self::RegisterActiveListenerStream>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn register_passive_listener(
        &self,
        request: Request<Streaming<EventStructure>>,
    ) -> Result<Response<RequestPushEvent>, Status> {
        debug!("Received passive listener registration");
        debug!("Request: {:?}", request.metadata());
        let uid = request
            .metadata()
            .get("kritor-self-uid")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        // wait until bot is created through reverse stream
        loop {
            if BOTS.read().await.contains_key(&uid) {
                break;
            }
            NOTIFY.notified().await;
        }
        debug!("Bot found: {}", uid);
        let bot = {
            let bots = BOTS.read().await;
            let bot = bots.get(&uid).unwrap();
            bot.clone()
        };
        let mut receiving_stream = request.into_inner();

        while let Some(event) = receiving_stream.next().await {
            match event {
                Ok(event) => {
                    let bot = bot.clone();
                    tokio::spawn(async move {
                        debug!("Received event: {:?}", event);
                        let bot_guard = bot.read().await;
                        match event.event.unwrap() {
                            Event::Message(message) => {
                                let sender = bot_guard.get_message_sender();
                                let _ = sender.send(message);
                            }
                            Event::Notice(notice) => {
                                let sender = bot_guard.get_notice_sender();
                                let _ = sender.send(notice);
                            }
                            Event::Request(request) => {
                                let sender = bot_guard.get_request_sender();
                                let _ = sender.send(request);
                            }
                        }
                    });
                }
                Err(err) => {
                    error!("Error: {:?}", err);
                }
            }
        }
        info!("Stream ended");
        return Ok(Response::new(RequestPushEvent { r#type: 0 }));
    }
}

#[derive(Debug, Default)]
pub struct ReverseListener {}

type ReverseResponseStream = Pin<Box<dyn Stream<Item = Result<common::Request, Status>> + Send>>;

#[tonic::async_trait]
impl ReverseService for ReverseListener {
    type ReverseStreamStream = ReverseResponseStream;

    async fn reverse_stream(
        &self,
        request: Request<Streaming<common::Response>>,
    ) -> Result<Response<Self::ReverseStreamStream>, Status> {
        debug!("Received reverse stream");
        debug!("Request: {:?}", request.metadata());
        let (tx, rx) = mpsc::channel(4096);
        let uid = request
            .metadata()
            .get("kritor-self-uid")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let version = request
            .metadata()
            .get("kritor-self-version")
            .map(|m| m.to_str().unwrap().to_string());
        let uin = request
            .metadata()
            .get("kritor-self-uin")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            .parse()
            .unwrap_or(0);
        // wait until bot is created
        {
            let bots = BOTS.write().await;
            if !bots.contains_key(&uid) {
                let tx_mutex = Arc::new(Some(tx));
                let bot = Bot::new(uin, uid.clone(), tx_mutex, version);
                let bot_ref = Arc::new(RwLock::new(bot));
                listen_to_events(Arc::clone(&bot_ref)).await;
                bots.insert(uid.clone(), Arc::clone(&bot_ref));
                NOTIFY.notify_waiters();
            }
        }
        let bot = {
            let bots = BOTS.read().await;
            let bot = bots.get(&uid).unwrap();
            bot.clone()
        };
        let mut in_stream = request.into_inner();
        let bot_clone = bot.clone();
        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(v) => {
                        let binding = bot_clone.clone();
                        tokio::spawn(async move {
                            debug!("Received: {:?}", v);
                            let bot_guard = binding.read().await;
                            let queue = bot_guard.get_request_queue().clone();
                            if let Some(tx) = queue.remove(&v.seq) {
                                tx.1.send(v).expect("queue not exists");
                            } else {
                                warn!("unknown response: cmd: {}, seq: {}", v.cmd, v.seq);
                            }
                        });
                    }
                    Err(err) => {
                        if let Some(io_err) = match_for_io_error(&err) {
                            if io_err.kind() == ErrorKind::BrokenPipe {
                                error!("\tclient disconnected: broken pipe");
                                break;
                            }
                        }
                        error!("error: {:?}", err);
                    }
                }
            }
            println!("\treverse_stream ended");
            BOTS.write().await.remove(&uid);
        });
        tokio::spawn(async move {
            // sleep(Duration::from_secs(3)).await;
            Bot::init(bot.clone()).await;
        });
        let out_stream = ReceiverStream::new(rx);

        Ok(Response::new(
            Box::pin(out_stream) as Self::ReverseStreamStream
        ))
    }
}

fn match_for_io_error(err_status: &Status) -> Option<&std::io::Error> {
    let mut err: &(dyn Error + 'static) = err_status;

    loop {
        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
            return Some(io_err);
        }

        // h2::Error do not expose std::io::Error with `source()`
        // https://github.com/hyperium/h2/pull/462
        if let Some(h2_err) = err.downcast_ref::<h2::Error>() {
            if let Some(io_err) = h2_err.get_io() {
                return Some(io_err);
            }
        }

        err = match err.source() {
            Some(err) => err,
            None => return None,
        };
    }
}
