use rusqlite::{Connection, Result, params};
use core_shared::{Task, TaskStatus, TimelineMemo, NoteSnapshot, TaskGraphCommand};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS note_snapshots (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            captured_at INTEGER NOT NULL,
            content     TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS tasks (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            label         TEXT NOT NULL,
            status        TEXT NOT NULL DEFAULT 'Todo',
            parent_id     INTEGER REFERENCES tasks(id),
            source_text   TEXT NOT NULL DEFAULT '',
            cumulative_ms INTEGER NOT NULL DEFAULT 0,
            created_at    INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS timeline_memos (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id     INTEGER NOT NULL REFERENCES tasks(id),
            event_type  TEXT NOT NULL,
            description TEXT NOT NULL,
            timestamp   INTEGER NOT NULL
        );
    ")
}

pub fn save_snapshot(conn: &Connection, content: &str) -> Result<NoteSnapshot> {
    let now = now_secs();
    conn.execute(
        "INSERT INTO note_snapshots (captured_at, content) VALUES (?1, ?2)",
        params![now, content],
    )?;
    Ok(NoteSnapshot { id: conn.last_insert_rowid(), captured_at: now, content: content.to_string() })
}

pub fn get_latest_snapshot(conn: &Connection) -> Result<Option<NoteSnapshot>> {
    let mut stmt = conn.prepare(
        "SELECT id, captured_at, content FROM note_snapshots ORDER BY id DESC LIMIT 1"
    )?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(Some(NoteSnapshot {
            id: row.get(0)?,
            captured_at: row.get(1)?,
            content: row.get(2)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_all_tasks(conn: &Connection) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, label, status, parent_id, source_text, cumulative_ms, created_at FROM tasks ORDER BY id"
    )?;
    let tasks = stmt.query_map([], |row| {
        let status_str: String = row.get(2)?;
        Ok(Task {
            id: row.get(0)?,
            label: row.get(1)?,
            status: TaskStatus::try_from(status_str.as_str())
                .unwrap_or(TaskStatus::Todo),
            parent_id: row.get(3)?,
            source_text: row.get(4)?,
            cumulative_ms: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;
    tasks.collect()
}

pub fn create_task(conn: &Connection, label: &str, parent_id: Option<i64>, source_text: &str) -> Result<Task> {
    let now = now_secs();
    conn.execute(
        "INSERT INTO tasks (label, status, parent_id, source_text, created_at) VALUES (?1, 'Todo', ?2, ?3, ?4)",
        params![label, parent_id, source_text, now],
    )?;
    Ok(Task {
        id: conn.last_insert_rowid(),
        label: label.to_string(),
        status: TaskStatus::Todo,
        parent_id,
        source_text: source_text.to_string(),
        cumulative_ms: 0,
        created_at: now,
    })
}

pub fn update_task_status(conn: &Connection, task_id: i64, status: &str) -> Result<()> {
    conn.execute(
        "UPDATE tasks SET status = ?1 WHERE id = ?2",
        params![status, task_id],
    )?;
    Ok(())
}

pub fn add_timeline_memo(conn: &Connection, task_id: i64, event_type: &str, description: &str) -> Result<TimelineMemo> {
    let now = now_secs();
    conn.execute(
        "INSERT INTO timeline_memos (task_id, event_type, description, timestamp) VALUES (?1, ?2, ?3, ?4)",
        params![task_id, event_type, description, now],
    )?;
    Ok(TimelineMemo {
        id: conn.last_insert_rowid(),
        task_id,
        event_type: event_type.to_string(),
        description: description.to_string(),
        timestamp: now,
    })
}

pub fn apply_command(conn: &Connection, cmd: &TaskGraphCommand) -> Result<()> {
    match cmd {
        TaskGraphCommand::CreateTask { label, parent_id, source_text } => {
            create_task(conn, label, *parent_id, source_text)?;
        }
        TaskGraphCommand::UpdateTask { id, status } => {
            if let Some(s) = status {
                update_task_status(conn, *id, s)?;
            }
        }
        TaskGraphCommand::AddMemo { task_id, event_type, description } => {
            add_timeline_memo(conn, *task_id, event_type, description)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn save_and_retrieve_snapshot() {
        let conn = in_memory_db();
        save_snapshot(&conn, "hello world").unwrap();
        let snap = get_latest_snapshot(&conn).unwrap().unwrap();
        assert_eq!(snap.content, "hello world");
    }

    #[test]
    fn create_and_get_task() {
        let conn = in_memory_db();
        let t = create_task(&conn, "Fix bug", None, "- [ ] Fix bug").unwrap();
        assert_eq!(t.label, "Fix bug");
        let tasks = get_all_tasks(&conn).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, t.id);
    }

    #[test]
    fn update_task_status_changes_status() {
        let conn = in_memory_db();
        let t = create_task(&conn, "Task A", None, "").unwrap();
        update_task_status(&conn, t.id, "InProgress").unwrap();
        let tasks = get_all_tasks(&conn).unwrap();
        assert_eq!(tasks[0].status, TaskStatus::InProgress);
    }

    #[test]
    fn apply_create_command() {
        let conn = in_memory_db();
        let cmd = TaskGraphCommand::CreateTask {
            label: "AI 구현".to_string(),
            parent_id: None,
            source_text: "- [ ] AI 구현".to_string(),
        };
        apply_command(&conn, &cmd).unwrap();
        let tasks = get_all_tasks(&conn).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].label, "AI 구현");
    }
}
