use std::collections::HashMap;
use std::fmt::Error;
use std::io;
use std::sync::Arc;
use futures::channel::oneshot;
use once_cell::sync::Lazy;
use prost::Message;
use rand::Rng;
use tokio::sync::Mutex;
use bytes::{Bytes, Buf};
use crate::kritor::server::kritor_proto::common::{Contact, Element};
use crate::kritor::server::kritor_proto::{common, SendMessageRequest, SendMessageResponse};
use crate::kritor::server::TX_GLOBAL;

pub static REQUEST_QUEUE: Lazy<Arc<Mutex<HashMap<u32, oneshot::Sender<common::Response>>>>> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub async fn send_request(request: common::Request) -> Result<common::Response, io::Error> {
    let (resp_tx, resp_rx) = oneshot::channel();

    let mut tx_guard = TX_GLOBAL.lock().await;
    if let Some(tx) =  tx_guard.as_ref() {
        // 发送请求
        REQUEST_QUEUE.lock().await.insert(request.seq, resp_tx);
        tx.send(Ok(request)).await.expect("Failed to send request");

        // 等待响应
        match resp_rx.await {
            Ok(response) => Ok(response),
            Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Failed to receive response")),
        }
    } else {
        Err(io::Error::new(io::ErrorKind::NotConnected, "Not connected"))
    }
}

fn get_seq() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}
pub async fn send_msg(segments: Vec<Element>, contact: Contact) -> Result<SendMessageResponse, Error> {
    let msg = SendMessageRequest {
        contact: Some(contact),
        elements: segments,
        retry_count: None,
        message_id: None,
        notice_id: None,
        request_id: None,
    };
    let response = send_request(common::Request {
        cmd: "MessageService.SendMessage".to_string(),
        seq: get_seq(),
        buf: msg.encode_to_vec(),
        no_response: false,
    }).await.expect("send error");
    let buf: Bytes = response.buf.into();
    let response = SendMessageResponse::decode(buf).unwrap();
    Ok(response)
}