mod bot;
mod kritor;
mod model;
mod service;
mod test;
mod utils;

use crate::kritor::server::kritor_proto::event_service_server::EventServiceServer;
use crate::kritor::server::kritor_proto::reverse_service_server::ReverseServiceServer;
use crate::kritor::server::{EventListener, ReverseListener};
use crate::model::config::{get_config_sync, notify_config_change};
use crate::service::external::javascript::service::register_js_plugins;
use once_cell::sync::Lazy;
use std::error::Error;
use tonic::transport::Server;

pub static LOG_INIT: Lazy<()> = Lazy::new(|| {
    let config = get_config_sync();
    let level = config.log_level.unwrap_or("info".to_string());
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level.as_str()))
        .init();
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
