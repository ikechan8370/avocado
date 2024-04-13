use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use async_trait::async_trait;
use boa_engine::Source;
use avocado_common::Event;
use crate::kritor::server::kritor_proto::common::{Contact, Scene};
use crate::service::external::javascript::loader::generate_context;
use crate::service::register::register_service;
use crate::service::service::{KritorContext, Matchable, Service};
use crate::utils::kritor::same_contact_and_sender;


struct ExternalJsService {
    entry_path: PathBuf,

}

#[async_trait]
impl Matchable for ExternalJsService {
    fn matches(&self, _context: KritorContext) -> bool {
        // match在js端判断，此处提条件返回true
        true
    }
}


#[async_trait]
impl Service for ExternalJsService {
    async fn process(&self, context: KritorContext) {
        let bot_arc = context.bot.clone();
        let bot = bot_arc.read().await;
        let group = bot.get_groups();
        let group = group.read().await;
        let friends = bot.get_friends();
        let friends = friends.read().await;

        let nickname =  bot.get_nickname().await;

        let elements = context.message.as_ref().cloned().map(|message| message.elements);

        let plugin_name = {
            let service_name = context.current_service_name.read().await;
            service_name.clone()
        };
        let (sender, contact) = {
            if let Some(messages) = context.message.as_ref() {
                (
                    context.message.clone().map(|message| message.sender.unwrap()),
                    context.message.clone().map(|message| message.contact.unwrap())
                )
            } else if let Some(notice) = context.notice.as_ref() {
                (
                    None,
                    // todo
                    Some(Contact {
                        scene: Scene::Group.into(),
                        peer: "".to_string(),
                        sub_peer: None,
                    })
                )
            } else if let Some(request) = context.request.as_ref() {
                (
                    None,
                    // todo
                    Some(Contact {
                        scene: Scene::Group.into(),
                        peer: "".to_string(),
                        sub_peer: None,
                    })
                )
            } else {
                (None, None)
            }
        };
        let mut boa_context = generate_context(&group, &friends, bot.get_uin().unwrap_or_default(), bot.get_uid().unwrap_or_default(),
                                               nickname.unwrap_or_default(),
                                               sender, contact,
                                               elements.unwrap_or_default(), plugin_name.unwrap_or("unknown".to_string()));

        let source = Source::from_filepath(self.entry_path.as_path()).unwrap();
        boa_context.eval(source).expect("external javascript plugin execute error");
    }
}

pub async fn register_js_plugins() {
    // read directories from plugins/js
    let dirs = fs::read_dir("plugins/js").unwrap();
    for dir in dirs {
        let dir = dir.unwrap();
        let path = dir.path();
        if path.is_dir() && !path.ends_with("def"){
            // 遍历下面的js文件
            let files = fs::read_dir(path).unwrap();
            for file in files {
                let file = file.unwrap();
                let path = file.path();
                path.is_file().then(|| {
                    let mut name = &path.file_name().unwrap().to_str().unwrap();
                    let plugin_name = name.split(".").next().unwrap();
                    let service = ExternalJsService {
                        entry_path: path.clone(),
                    };
                    let service_arc = Arc::new(service);
                    register_service(service_arc, vec![Event::Message, Event::Notice, Event::Request], plugin_name.to_string());
                } );

            }
        }
    }
    // todo hot reload
}