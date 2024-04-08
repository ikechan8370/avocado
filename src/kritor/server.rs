use std::error::Error;
use std::io::ErrorKind;
use std::pin::Pin;
use std::sync::Arc;
use log::{error, info};
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use kritor_proto::event_service_server::EventService;
use kritor_proto::{common, EventStructure, RequestPushEvent, EventType};
use kritor_proto::reverse_service_server::ReverseService;
use tokio::sync::broadcast;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use crate::kritor::api::REQUEST_QUEUE;
use crate::kritor::server::kritor_proto::notice_event::Notice;
use crate::kritor::server::kritor_proto::{NoticeEvent, RequestEvent};
use crate::kritor::server::kritor_proto::event_structure::Event;

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


pub static MESSAGE_CHANNEL: Lazy<Mutex<(broadcast::Sender<common::PushMessageBody>, broadcast::Receiver<common::PushMessageBody>)>> = Lazy::new(|| {
    let (sender, receiver) = broadcast::channel(100);
    Mutex::new((sender, receiver))
});

pub static NOTICE_CHANNEL: Lazy<Mutex<(broadcast::Sender<NoticeEvent>, broadcast::Receiver<NoticeEvent>)>> = Lazy::new(|| {
    let (sender, receiver) = broadcast::channel(100);
    Mutex::new((sender, receiver))
});

pub static REQUEST_CHANNEL: Lazy<Mutex<(broadcast::Sender<RequestEvent>, broadcast::Receiver<RequestEvent>)>> = Lazy::new(|| {
    let (sender, receiver) = broadcast::channel(100);
    Mutex::new((sender, receiver))
});

pub static TX_GLOBAL: Lazy<Arc<Mutex<Option<mpsc::Sender<Result<common::Request, Status>>>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

#[derive(Debug, Default)]
pub struct EventListener {}

type ResponseStream = Pin<Box<dyn Stream<Item = Result<EventStructure, Status>> + Send>>;

#[tonic::async_trait]
impl EventService for EventListener {
    type RegisterActiveListenerStream = ResponseStream;

    async fn register_active_listener(&self, request: Request<RequestPushEvent>) -> Result<Response<Self::RegisterActiveListenerStream>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn register_passive_listener(&self, request: Request<Streaming<EventStructure>>) -> Result<Response<RequestPushEvent>, Status> {
        info!("Received passive listener registration");
        info!("Request: {:?}", request.metadata());
        let mut receiving_stream = request.into_inner();

        while let Some(event) = receiving_stream.next().await {
            match event {
                Ok(event) => {
                    match event.event.unwrap() {
                        Event::Message(message) => {
                            let sender = &MESSAGE_CHANNEL.lock().await.0;
                            let _ = sender.send(message);
                        }
                        Event::Notice(notice) => {
                            let sender = &NOTICE_CHANNEL.lock().await.0;
                            let _ = sender.send(notice);
                        }
                        Event::Request(request) => {
                            let sender = &REQUEST_CHANNEL.lock().await.0;
                            let _ = sender.send(request);
                        }
                    }
                }
                Err(err) => {
                    error!("Error: {:?}", err);
                }
            }
        }
        info!("Stream ended");
        return Ok(Response::new(RequestPushEvent {
            r#type: 0,
        }));
    }
}

#[derive(Debug, Default)]
pub struct ReverseListener {}

type ReverseResponseStream = Pin<Box<dyn Stream<Item = Result<common::Request, Status>> + Send>>;

#[tonic::async_trait]
impl ReverseService for ReverseListener {
    type ReverseStreamStream = ReverseResponseStream;

    async fn reverse_stream(&self, request: Request<Streaming<common::Response>>) -> Result<Response<Self::ReverseStreamStream>, Status> {
        info!("Received reverse stream");
        info!("Request: {:?}", request.metadata());
        // let response = request.into_inner().message().await.unwrap().unwrap();
        // info!("response: {:?}", response);

        let mut in_stream = request.into_inner();

        let (tx, rx) = mpsc::channel(128);
        *TX_GLOBAL.lock().await = Some(tx);

        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(v) => {
                        info!("receive response: {:?}", v);
                        let mut queue = REQUEST_QUEUE.lock().await;
                        let tx = queue.remove(&v.seq).unwrap();
                        tx.send(v).expect("queue not exists");
                    },
                    Err(err) => {
                        if let Some(io_err) = match_for_io_error(&err) {
                            if io_err.kind() == ErrorKind::BrokenPipe {
                                // here you can handle special case when client
                                // disconnected in unexpected way
                                eprintln!("\tclient disconnected: broken pipe");
                                break;
                            }
                        }
                        error!("error: {:?}", err);
                    }
                }
            }
            println!("\treverse_stream ended");
        });

        // echo just write the same data that was received
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