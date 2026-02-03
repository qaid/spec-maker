use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute_batch(include_str!("schema.sql"))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

pub mod models {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Project {
        pub id: String,
        pub name: String,
        pub description: String,
        pub industry: Option<String>,
        pub target_audience: Option<String>,
        pub status: String,
        pub created_at: String,
        pub updated_at: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateProjectInput {
        pub name: String,
        pub description: String,
        pub industry: Option<String>,
        pub target_audience: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Conversation {
        pub id: String,
        pub project_id: String,
        pub phase: String,
        pub created_at: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Message {
        pub id: String,
        pub conversation_id: String,
        pub role: String,
        pub content: String,
        pub metadata: Option<String>,
        pub created_at: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateMessageInput {
        pub conversation_id: String,
        pub role: String,
        pub content: String,
        pub metadata: Option<String>,
    }
}
