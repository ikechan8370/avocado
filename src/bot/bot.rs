use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicI32;
use std::time::{SystemTime, UNIX_EPOCH};
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
use crate::bot::friend::{Friend, FriendAPITrait};
use crate::bot::group::{Group, GroupAPITrait};
use crate::kritor::server::kritor_proto::*;
use crate::{err, kritor_err};
use crate::bot::core::CoreAPITrait;
use crate::kritor::server::kritor_proto::common::{Contact, Element};
use crate::service::service::KritorContext;
use crate::utils::kritor::same_contact_and_sender;

#[derive(Debug)]
pub struct Bot {
    message_sender: broadcast::Sender<common::PushMessageBody>,
    notice_sender: broadcast::Sender<NoticeEvent>,
    request_sender: broadcast::Sender<RequestEvent>,
    request_queue: Arc<DashMap<u32, oneshot::Sender<common::Response>>>,
    response_listener: Arc<Option<Sender<Result<common::Request, Status>>>>,
    uin: Option<u64>,
    uid: Option<String>,
    nickname: Arc<RwLock<Option<String>>>,
    groups: Arc<RwLock<Option<HashMap<u64, Group>>>>,
    friends: Arc<RwLock<Option<HashMap<u64, Friend>>>>,
    client_version: Arc<RwLock<Option<String>>>,
    kritor_version: Option<String>,
    // 启动时间时间戳 单位秒
    up_time: u64,
    // 已经发送的消息数
    sent: AtomicI32,
    // 接收的消息数
    receive: AtomicI32,
    // context transaction lock, when it exists for a contact, message won't be sent to handlers for the contact
    // 对于每个contact是唯一的，也就是每个人同时最多只能进行一个trans
    transaction_contexts: Arc<RwLock<Vec<(KritorContext, Contact, common::Sender)>>>,
}

impl Bot {
    pub fn new(uin: u64, uid: String, tx_mutex: Arc<Option<Sender<Result<common::Request, Status>>>>, version: Option<String>) -> Self {
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
            nickname: Arc::new(RwLock::new(None)),
            groups: Arc::new(RwLock::new(Some(HashMap::new()))),
            uin: Some(uin),
            friends: Arc::new(RwLock::new(Some(HashMap::new()))),

            client_version: Arc::new(RwLock::new(None)),
            kritor_version: version,

            up_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            sent: AtomicI32::new(0),
            receive: AtomicI32::new(0),

            transaction_contexts: Arc::new(RwLock::new(vec![])),
        }
    }

    pub fn get_request_queue(&self) -> &Arc<DashMap<u32, oneshot::Sender<common::Response>>> {
        &self.request_queue
    }

    pub async fn init(self_arc: Arc<RwLock<Self>>) {
        {
            let self_guard = self_arc.read().await;
            // let version = self_guard.get_version().await.expect("Failed to get version");
            // self_guard.client_version.write().await.replace(version.clone().app_name);
            // info!("Client version: {}", version.app_name);
            self_guard.client_version.write().await.replace("Android QQ".to_string());
            let current = self_guard.get_current_account().await.expect("Failed to get nickname");
            self_guard.nickname.write().await.replace(current.account_name.clone());
            info!("Welcome Nickname: {}", current.account_name);
        }

        let self_guard = self_arc.read().await;
        info!("Start to get friends list");
        let result = self_guard.get_friend_list(true).await;
        drop(self_guard);
        match result {
            Ok(friends_response) => {
                let info = friends_response.friends_info.clone();
                let friend_num = info.len();
                info!("Friends count: {}", friend_num);
                let friends = info.into_iter().map(|info| {
                    let friend = Friend::new(info);
                    (friend.inner.uin, friend)
                }).collect::<HashMap<u64, Friend>>();
                let self_guard = self_arc.write().await;
                let mut final_friends = self_guard.friends.write().await;
                final_friends.get_or_insert_with(HashMap::new).extend(friends);
            }
            Err(err) => {
                error!("Failed to initialize friends: {:?}", err.error());
            }
        }

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
                        let group_members_info = &self_guard.get_group_member_list(group_info.group_id, true).await.expect("Failed to get group member list").group_members_info;
                        debug!("group member list: {:?}", group_info.group_id);
                        (group_info.group_id, group_members_info.clone())
                    }
                }).collect::<FuturesUnordered<_>>(); // 使用 FuturesUnordered 来处理并发
                let results: HashMap<u64, Vec<GroupMemberInfo>> = tasks.collect().await;

                debug!("prepare to update bot");
                let self_guard = self_arc.write().await;
                let mut final_groups = self_guard.groups.write().await;
                for group_info in &groups.groups_info {
                    let group_id = group_info.group_id;
                    let group_members_info = results.get(&group_id).expect("Failed to get group members info");
                    let group_members_map = group_members_info.iter().map(|member_info| {
                        (member_info.uin, member_info.clone())
                    }).collect::<HashMap<u64, GroupMemberInfo>>();
                    let group = Group::new(group_info.clone(), group_members_map);
                    final_groups.get_or_insert_with(HashMap::new).insert(group_id, group);
                }
            }
            Err(err) => {
                error!("Failed to initialize groups: {:?}", err.error());
            }
        };


        info!("Bot initialized");
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

    pub fn get_kritor_version(&self) -> Option<String> {
        self.kritor_version.clone()
    }

    pub async fn get_client_version(&self) -> Option<String> {
        let arc = self.client_version.clone();
        let lock = arc.read().await;
        lock.clone()
    }

    pub fn get_uin(&self) -> Option<u64> {
        self.uin
    }

    pub fn get_uid(&self) -> Option<String> {
        self.uid.clone()
    }
    pub async fn get_nickname(&self) -> Option<String> {
        let arc = self.nickname.clone();
        let lock = arc.read().await;
        lock.clone()
    }

    pub fn get_uptime(&self) -> u64 {
        self.up_time
    }

    pub fn get_sent(&self) -> i32 {
        self.sent.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn plus_one_sent(&self) {
        self.plus_sent(1)
    }

    pub fn plus_sent(&self, delta: i32) {
        self.sent.fetch_add(delta, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn get_receive(&self) -> i32 {
        self.receive.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn plus_one_receive(&self) {
        self.plus_receive(1)
    }

    pub fn plus_receive(&self, delta: i32) {
        self.receive.fetch_add(delta, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_groups_arc(&self) -> Arc<RwLock<Option<HashMap<u64, Group>>>> {
        self.groups.clone()
    }

    pub async fn get_groups(&self) -> Option<HashMap<u64, Group>> {
        let guard = self.groups.read().await;
        guard.clone()
    }

    pub fn get_friends_arc(&self) -> Arc<RwLock<Option<HashMap<u64, Friend>>>> {
        self.friends.clone()
    }

    pub async fn get_friends(&self) -> Option<HashMap<u64, Friend>> {
        let guard = self.friends.read().await;
        guard.clone()
    }

    /// 对指定的contact停止广播，用于开始trans的情况
    pub async fn stop_broadcast(self_arc: Arc<RwLock<Self>>, context: KritorContext, contact: Contact, sender: common::Sender) -> crate::model::error::Result<()> {
        {
            let self_guard = self_arc.read().await;
            let mut lock = self_guard.transaction_contexts.write().await;
            lock.push((context, contact, sender));
        }
        Ok(())
    }

    /// 恢复对指定contact的广播
    pub async fn resume_broadcast(self_arc: Arc<RwLock<Self>>, contact: Contact, sender: common::Sender) -> crate::model::error::Result<()> {
        let self_guard = self_arc.read().await;
        let mut lock = self_guard.transaction_contexts.write().await;
        // let (context, c) = lock.clone().iter().find(|(ctx, c)| c == &contact).ok_or_else(|| client_err!("Transaction is not locked"))?;
        // 恢复到该contact的广播
        lock.clone().iter().position(|(ctx, c, s)| same_contact_and_sender((&contact, &sender), (c, s))).map(|pos| lock.remove(pos));
        Ok(())
    }


    pub async fn get_broadcast_lock(&self) -> Arc<RwLock<Vec<(KritorContext, Contact, common::Sender)>>> {
        self.transaction_contexts.clone()
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
        let buf: Bytes = response.buf.clone().into();
        let response = SendMessageResponse::decode(buf).unwrap();
        self.plus_one_sent();
        Ok(response)
    }
}

pub fn get_seq() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}