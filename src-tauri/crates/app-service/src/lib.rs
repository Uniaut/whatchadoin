//! UI 요청 처리 레이어.
//! SQLite DB를 보유하고 Tauri 커맨드에서 호출된다.

pub mod db;

use rusqlite::Connection;
use std::sync::Mutex;
use core_shared::{Task, TaskGraphCommand};

pub struct AppService {
    pub db: Mutex<Connection>,
}

impl AppService {
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        db::init_schema(&conn)?;
        Ok(Self { db: Mutex::new(conn) })
    }

    pub fn save_snapshot(&self, content: &str) -> Result<Option<String>, rusqlite::Error> {
        let conn = self.db.lock().unwrap();
        let prev = db::get_latest_snapshot(&conn)?;
        db::save_snapshot(&conn, content)?;
        Ok(prev.map(|s| s.content))
    }

    pub fn apply_commands(&self, cmds: &[TaskGraphCommand]) -> Result<(), rusqlite::Error> {
        let conn = self.db.lock().unwrap();
        for cmd in cmds {
            db::apply_command(&conn, cmd)?;
        }
        Ok(())
    }

    pub fn get_tasks(&self) -> Result<Vec<Task>, rusqlite::Error> {
        let conn = self.db.lock().unwrap();
        db::get_all_tasks(&conn)
    }

    pub fn update_task_status(&self, task_id: i64, status: &str) -> Result<(), rusqlite::Error> {
        let conn = self.db.lock().unwrap();
        db::update_task_status(&conn, task_id, status)
    }
}
