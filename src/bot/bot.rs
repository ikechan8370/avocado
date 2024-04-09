use std::collections::HashMap;
use std::sync::Arc;
use bytes::Bytes;
use dashmap::DashMap;
use futures::channel::oneshot;
use futures::StreamExt;
use futures::future::join_all;
use futures::stream::FuturesUnordered;
use log::{debug, error, info};
use prost::Message;
use rand::Rng;
use tokio::sync::{broadcast, mpsc, Mutex, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tonic::Status;
use crate::bot::friend::Friend;
use crate::bot::group::{Group, GroupAPITrait};
use crate::kritor::server::kritor_proto::{common, GetGroupInfoRequest, GetGroupInfoResponse, GetGroupListRequest, GetGroupListResponse, GetGroupMemberInfoRequest, GetGroupMemberInfoResponse, GetGroupMemberListResponse, GroupInfo, GroupMemberInfo, NoticeEvent, RequestEvent, SendMessageRequest, SendMessageResponse};
use crate::{err, kritor_err};
use crate::kritor::server::kritor_proto::common::{Contact, Element};
use crate::kritor::server::kritor_proto::get_group_member_info_request::Target;
use crate::model::error::Error;

#[derive(Debug)]
pub struct Bot {
    message_sender: broadcast::Sender<common::PushMessageBody>,
    message_receiver: Mutex<broadcast::Receiver<common::PushMessageBody>>,
    notice_sender: broadcast::Sender<NoticeEvent>,
    notice_receiver: Mutex<broadcast::Receiver<NoticeEvent>>,
    request_sender: broadcast::Sender<RequestEvent>,
    request_receiver: Mutex<broadcast::Receiver<RequestEvent>>,
    request_queue: RwLock<DashMap<u32, oneshot::Sender<common::Response>>>,
    response_listener: RwLock<Option<mpsc::Sender<Result<common::Request, Status>>>>,
    uin: Option<u64>,
    uid: Option<String>,
    nickname: Option<String>,
    groups: Option<HashMap<u64, Group>>,
    friends: Option<HashMap<u64, Friend>>
}

impl Bot {
    pub fn new(uin: u64, uid: String) -> Self {
        info!("Bot is created: uin: {}, uid: {}", uin, uid);
        let (msender, mreceiver) = broadcast::channel(100);
        let (nsender, nreceiver) = broadcast::channel(100);
        let (rsender, rreceiver) = broadcast::channel(100);
        Self {
            message_sender: msender,
            message_receiver: Mutex::new(mreceiver),
            notice_sender: nsender,
            notice_receiver: Mutex::new(nreceiver),
            request_sender: rsender,
            request_receiver: Mutex::new(rreceiver),
            request_queue: RwLock::new(DashMap::new()),
            response_listener: RwLock::new(None),
            uid: Some(uid),
            nickname: None,
            groups: Some(HashMap::new()),
            uin: Some(uin),
            friends: Some(HashMap::new()),
        }
    }

    pub fn get_request_queue(&self) -> &RwLock<DashMap<u32, oneshot::Sender<common::Response>>> {
        &self.request_queue
    }
    pub async fn set_response_listener(&mut self, listener: RwLock<Option<mpsc::Sender<Result<common::Request, Status>>>>) {
        self.response_listener = listener;
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

        {
            let tx_guard = self.response_listener.read().await;
            if let Some(tx) = tx_guard.as_ref() {
                {
                    let mut request_queue = self.request_queue.read().await;
                    request_queue.insert(request.seq, resp_tx);
                }

                tx.send(Ok(request.clone())).await.expect("Failed to send request");
            } else {
                return err!("Connection not established");
            }
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