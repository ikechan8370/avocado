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

pub static LOG_INIT: Lazy<()> = Lazy::new(|| {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

});

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "0.0.0.0:7001".parse()?;
    let event_listener = EventListener::default();
    let reverse_listener = ReverseListener::default();
    Server::builder()
        .add_service(EventServiceServer::new(event_listener))
        .add_service(ReverseServiceServer::new(reverse_listener))
        .serve(addr)
        .await?;
    Ok(())
}

