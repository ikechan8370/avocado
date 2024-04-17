use std::path::Path;
use std::sync::Arc;
use once_cell::sync::Lazy;
use rusqlite::Connection;

#[derive(Debug)]
struct Store {
    key: String,
    value: String,
    namespace: Namespace,
}

pub type Namespace = String;

pub const DEFAULT_NAMESPACE: &str = "default";

impl Default for Store {
    fn default() -> Self {
        Store {
            key: String::new(),
            value: String::new(),
            namespace: "default".to_string(),
        }
    }
}

impl Store {
    fn key(&mut self, key: String) -> &mut Self {
        self.key = key;
        self
    }

    fn value(&mut self, value: String) -> &mut Self {
        self.value = value;
        self
    }
}

pub struct StoreManager {
    conn: Connection,
}

impl StoreManager {
    pub fn new() -> Self {
        let conn = Connection::open(Path::new("data.db")).unwrap();
        // check if table avocado_store exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS avocado_store (
                key TEXT,
                value TEXT,
                namespace TEXT,
                PRIMARY KEY (key, namespace)
            )",
            [],
        ).unwrap();
        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_key_namespace ON avocado_store(key, namespace)",
            [],
        ).unwrap();
        StoreManager {
            conn,
        }
    }

    pub fn get(&self, key: &str, namespace: Option<Namespace>) -> Option<String> {
        let namespace = namespace.unwrap_or(DEFAULT_NAMESPACE.to_string());
        let mut stmt = self.conn.prepare("SELECT key, value, namespace FROM avocado_store WHERE key = ? AND namespace = ?").unwrap();
        let mut rows = stmt.query([key.to_string(), namespace as String]).unwrap();
        let row = rows.next().unwrap()?;
        let value: String = row.get(1).unwrap();
        Some(value)
    }

    pub fn set(&self, key: &str, value: String, namespace: Option<Namespace>) -> crate::model::error::Result<()> {
        let namespace = namespace.unwrap_or(DEFAULT_NAMESPACE.to_string());
        upsert_into_avocado_store(&self.conn, &key, &value, &namespace)?;
        Ok(())
    }

    pub fn delete(&self, key: &str, namespace: Option<Namespace>) -> crate::model::error::Result<()> {
        let namespace = namespace.unwrap_or(DEFAULT_NAMESPACE.to_string());
        self.conn.execute("DELETE FROM avocado_store WHERE key = ? AND namespace = ?", [key.to_string(), namespace])?;
        Ok(())
    }
}


fn upsert_into_avocado_store(conn: &Connection, key: &str, value: &str, namespace: &str) -> crate::model::error::Result<()> {
    // 尝试插入数据，如果已存在则忽略
    let result = conn.execute(
        "INSERT OR IGNORE INTO avocado_store (key, value, namespace) VALUES (?, ?, ?)",
        &[key, value, namespace],
    )?;

    if result == 0 {
        // 数据已存在，执行更新操作
        conn.execute(
            "UPDATE avocado_store SET value = ? WHERE key = ? AND namespace = ?",
            &[value, key, namespace],
        )?;
    }

    Ok(())
}

pub const STORE: Lazy<Arc<StoreManager>> = Lazy::new(|| Arc::new(StoreManager::new()));