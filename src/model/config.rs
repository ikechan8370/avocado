use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use futures::{SinkExt, StreamExt};
use log::{debug, error, info};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use serde::Deserialize;
use tokio::sync::RwLock;
use crate::model::error::Result;
use crate::err;
use crate::service::register::{INITIALIZED, RUNTIME};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub owner: Option<Vec<String>>,
    pub log_level: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            owner: None,
            log_level: Some("info".to_string()),
        }
    }
}

pub const CONFIG_PATH: &'static str = "config/config.toml";

fn read_toml_config() -> Result<Config> {
    let mut content = String::new();
    fs::File::open(CONFIG_PATH)?.read_to_string(&mut content)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

pub static GLOBAL_CONFIG: Lazy<Arc<RwLock<Config>>> = Lazy::new(|| Arc::new(RwLock::new(read_toml_config().unwrap())));

pub fn notify_config_change() {
    tokio::spawn(async move {
        let (mut tx, mut rx) = tokio::sync::mpsc::channel(1);
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                futures::executor::block_on(async {
                    tx.send(res).await.unwrap();
                })
            },
            notify::Config::default()
                .with_poll_interval(Duration::from_secs(2))
                .with_compare_contents(true),
        ).unwrap();

        let config_dir = fs::read_dir("config").unwrap();
        for file in config_dir {
            let file = file.unwrap();
            let path = file.path();
            if path.is_file() && path.extension().unwrap_or_default().to_str().unwrap_or_default().to_string() == "toml" {
                debug!("watch config file: {:?}", path);
                watcher.watch(path.as_path(), RecursiveMode::Recursive).unwrap();
            }
        }

        while let Some(event) = rx.recv().await {
            match event {
                Ok(event) => {
                    debug!("config file changed: {:?}", event);
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) => {
                            match read_toml_config() {
                                Ok(new_config) => {
                                    let mut config = GLOBAL_CONFIG.write().await;
                                    *config = new_config;
                                    info!("Config updated: {:?}", *config);
                                },
                                Err(e) => {
                                    error!("Failed to read new config: {}", e);
                                },
                            }
                        }
                        _ => {}
                    }
                },
                Err(e) => error!("watch error: {:?}", e),
            }
        }
    });
}

/// 从内存读取配置
pub async fn get_config() -> Config {
    GLOBAL_CONFIG.read().await.clone()
}


/// 直接读取本地文件，效率略低
pub fn get_config_sync() -> Config {
    read_toml_config().unwrap()
}