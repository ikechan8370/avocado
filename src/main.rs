mod kritor;
mod service;
mod model;
mod bot;
mod utils;
mod test;

use std::error::Error;
use once_cell::sync::Lazy;
use tonic::{transport::Server};
use crate::kritor::server::{EventListener, ReverseListener};
use crate::kritor::server::kritor_proto::event_service_server::EventServiceServer;
use crate::kritor::server::kritor_proto::reverse_service_server::ReverseServiceServer;
use crate::model::config::{get_config_sync, notify_config_change};
use crate::service::external::javascript::service::register_js_plugins;

pub static LOG_INIT: Lazy<()> = Lazy::new(|| {
    let config = get_config_sync();
    let level = config.log_level.unwrap_or("info".to_string());
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level.as_str())).init();
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    console_subscriber::init();
    let addr = "0.0.0.0:7001".parse()?;
    register_js_plugins().await;
    notify_config_change();
    let event_listener = EventListener::default();
    let reverse_listener = ReverseListener::default();
    Server::builder()
        .add_service(EventServiceServer::new(event_listener))
        .add_service(ReverseServiceServer::new(reverse_listener))
        .serve(addr)
        .await?;
    Ok(())
}

