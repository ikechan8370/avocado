use std::time::SystemTime;
use std::vec;

use async_trait::async_trait;
use avocado_common::Event;
use avocado_macro::service;

use crate::kritor::server::kritor_proto::common::{Scene};
use crate::service::service::{KritorContext, Service};
use crate::text;
use crate::utils::time::format_duration;

#[derive(Debug, Clone, Default)]
#[service(
    name = "status",
    pattern = "^([!！])(status|Status|STATUS|状态)$",
    events(Event::Message)
)]
struct StatusService;


#[async_trait]
impl Service for StatusService {
    // fn matches(&self, context: KritorContext) -> bool {
    //     let re = Regex::new(r"^([!！])(status|Status|STATUS|状态)$").unwrap();
    //     if let Some(message) = context.message {
    //         if let Some(elements) = message.elements.get_text_elements() {
    //             return elements.iter().any(|ele| re.is_match(ele.text.as_str()));
    //         }
    //     }
    //     false
    // }

    async fn process(&self, context: KritorContext) {
        let text = {
            let bot = context.bot.read().await;
            let nickname = context.message.as_ref().and_then(|m| m.sender.as_ref().and_then(|s| s.nick.as_ref())).cloned().unwrap_or_default();
            let uin = context.message.as_ref().and_then(|m| m.sender.as_ref().and_then(|s| s.uin.as_ref())).cloned().unwrap_or_default();
            let uid = context.message.as_ref().and_then(|m| m.sender.as_ref().map(|s| s.uid.as_str())).unwrap_or_default();

            let more = match context.message.as_ref().and_then(|m| m.contact.as_ref().map(|c| c.scene())) {
                Some(Scene::Group) => format!("群号: {}", context.message.as_ref().unwrap().contact.as_ref().unwrap().peer),
                Some(Scene::Friend) => format!("私聊对象: {}", context.message.as_ref().unwrap().contact.as_ref().unwrap().peer),
                _ => "".to_string(),
            };
            let mut text = format!("发送者信息\nnickname: {}\nuin: {}\nuid: {}\n{}", nickname, uin, uid, more);

            let start_time = bot.get_uptime();
            let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() - start_time;
            let duration_str = format_duration(duration).unwrap_or("刚刚启动".to_string());

            let version = bot.get_kritor_version().unwrap_or("未知".to_string());
            let groups =  bot.get_groups();
            let groups = groups.read().await;
            let group_num = groups.as_ref().unwrap().len();
            let friends = bot.get_friends();
            let friends = friends.read().await;
            let friend_num = friends.as_ref().unwrap().len();

            text = text + format!("\n\n运行状态\n运行时间：{}\n协议版本：{}\n已发送：{}\n已接收：{}\n群总数：{}\n好友总数：{}\n\nPowered by avocado-rs and kritor with ❤",
                                  duration_str, version, bot.get_sent(), bot.get_receive(), group_num, friend_num).as_str();
            text
        };
        context.reply_with_quote(vec![text!(text)]).await.unwrap();
    }
}