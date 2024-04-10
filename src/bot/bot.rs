use std::collections::HashMap;
use std::sync::Arc;
use bytes::Bytes;
use dashmap::DashMap;
use futures::channel::oneshot;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use log::{debug, error, info};
use prost::Message;
use rand::Rng;
use tokio::sync::{broadcast, RwLock, Semaphore};
use tokio::sync::mpsc::Sender;
use tonic::Status;
use crate::bot::friend::Friend;
use crate::bot::group::{Group, GroupAPITrait};
use crate::kritor::server::kritor_proto::*;
use crate::{err, kritor_err};
use crate::kritor::server::kritor_proto::common::{Contact, Element};

#[derive(Debug)]
pub struct Bot {
    message_sender: broadcast::Sender<common::PushMessageBody>,
    notice_sender: broadcast::Sender<NoticeEvent>,
    request_sender: broadcast::Sender<RequestEvent>,
    request_queue: Arc<DashMap<u32, oneshot::Sender<common::Response>>>,
    response_listener: Arc<Option<Sender<Result<common::Request, Status>>>>,
    uin: Option<u64>,
    uid: Option<String>,
    nickname: Option<String>,
    groups: Option<HashMap<u64, Group>>,
    friends: Option<HashMap<u64, Friend>>
}

impl Bot {
    pub fn new(uin: u64, uid: String, tx_mutex: Arc<Option<Sender<Result<common::Request, Status>>>>) -> Self {
        info!("Bot is created: uin: {}, uid: {}", uin, uid);
        let (msender, _mreceiver) = broadcast::channel(100);
        let (nsender, _nreceiver) = broadcast::channel(100);
        let (rsender, _rreceiver) = broadcast::channel(100);
        Self {
            message_sender: msender,
            notice_sender: nsender,
            request_sender: rsender,
            request_queue: Arc::new(DashMap::new()),
            response_listener: tx_mutex,
            uid: Some(uid),
            nickname: None,
            groups: Some(HashMap::new()),
            uin: Some(uin),
            friends: Some(HashMap::new()),
        }
    }

    pub fn get_request_queue(&self) -> &Arc<DashMap<u32, oneshot::Sender<common::Response>>> {
        &self.request_queue
    }

    pub async fn init(self_arc: Arc<RwLock<Self>>) {
        let self_guard = self_arc.read().await;
        let max_concurrent = 20;
        info!("Start to get group list");
        let result = self_guard.get_group_list(true).await;
        drop(self_guard);
        match result {
            Ok(groups) => {
                debug!("group list: {:?}", groups);
                let semaphore = Arc::new(Semaphore::new(max_concurrent));
                debug!("Start to get group member list");
                let tasks = groups.groups_info.clone().into_iter().map(|group_info| {
                    let semaphore_clone = semaphore.clone();
                    let bot_clone = self_arc.clone();
                    async move {
                        let _permit = semaphore_clone.acquire_owned().await.expect("Failed to acquire semaphore");
                        let self_guard = bot_clone.read().await;
                        let group_members_info = self_guard.get_group_member_list(group_info.group_id, true).await.expect("Failed to get group member list").group_members_info;
                        debug!("group member list: {:?}", group_info.group_id);
                        (group_info.group_id, group_members_info)
                    }
                }).collect::<FuturesUnordered<_>>(); // 使用 FuturesUnordered 来处理并发
                let results: HashMap<u64, Vec<GroupMemberInfo>> = tasks.collect().await;

                debug!("prepare to update bot");
                let mut self_guard = self_arc.write().await;
                groups.groups_info.into_iter().for_each(|group_info| {
                    let group_id = group_info.group_id;
                    let group_members_info = results.get(&group_id).expect("Failed to get group members info");
                    let group_members_map = group_members_info.iter().map(|member_info| {
                        (member_info.uin, member_info.clone())
                    }).collect::<HashMap<u64, GroupMemberInfo>>();
                    let group = Group::new(group_info, group_members_map);
                    self_guard.groups.get_or_insert_with(HashMap::new).insert(group_id, group);
                });
                info!("Bot initialized");
            }
            Err(err) => {
                error!("Failed to initialize bot: {:?}", err.error());
            }
        } ;

    }
    pub fn get_message_sender(&self) -> &broadcast::Sender<common::PushMessageBody> {
        &self.message_sender
    }

    pub fn subscribe_message(&self) -> broadcast::Receiver<common::PushMessageBody> {
        self.message_sender.subscribe()
    }

    pub fn get_notice_sender(&self) -> &broadcast::Sender<NoticeEvent> {
        &self.notice_sender
    }

    pub fn subscribe_notice(&self) -> broadcast::Receiver<NoticeEvent> {
        self.notice_sender.subscribe()
    }

    pub fn get_request_sender(&self) -> &broadcast::Sender<RequestEvent> {
        &self.request_sender
    }

    pub fn subscribe_request(&self) -> broadcast::Receiver<RequestEvent> {
        self.request_sender.subscribe()
    }

    pub async fn send_request(&self, request: common::Request) -> crate::model::error::Result<common::Response> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let tx_guard = self.response_listener.clone();

        if let Some(tx) = tx_guard.as_ref() {
            let request_queue = self.request_queue.clone();
            request_queue.insert(request.seq, resp_tx);
            tx.send(Ok(request.clone())).await.expect("Failed to send request");
        } else {
            return err!("Connection not established");
        }

        debug!("Request sent: {:?}", request);

        match resp_rx.await {
            Ok(response) => {
                debug!("Response received, cmd: {}, seq: {}", response.cmd, response.seq);
                Ok(response)
            },
            Err(e) => kritor_err!(format!("Failed to receive response: {}", e)),
        }
    }


    pub async fn send_msg(&self, segments: Vec<Element>, contact: Contact) -> crate::model::error::Result<SendMessageResponse> {
        let msg = SendMessageRequest {
            contact: Some(contact),
            elements: segments,
            retry_count: None,
            message_id: None,
            notice_id: None,
            request_id: None,
        };
        let response = self.send_request(common::Request {
            cmd: "MessageService.SendMessage".to_string(),
            seq: get_seq(),
            buf: msg.encode_to_vec(),
            no_response: false,
        }).await.expect("send error");
        let buf: Bytes = response.buf.into();
        let response = SendMessageResponse::decode(buf).unwrap();
        Ok(response)
    }
}

pub fn get_seq() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}