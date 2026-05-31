# AI Agent Pipeline Implementation Plan (Plan A)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 사용자가 마크다운 노트를 자유롭게 작성하면, 15초마다 AI 에이전트 파이프라인이 diff를 분석해 내부 task graph를 자동으로 유지하고, 칸반 UI로 표시한다.

**Architecture:** WorkView(FE) → 15s invoke("save_snapshot") → Rust: diff 감지 → Agent1(Diff Parser, Claude API) → Agent2(Task Mapper, Claude API) → SQLite task graph 업데이트 → FE emit("task-graph-updated") → TaskKanban 갱신. 노트는 사용자 영역, task graph는 AI 내부 영역.

**Tech Stack:** Tauri 2 (Rust), React 19 (TypeScript), rusqlite 0.31 (bundled), ureq 2 (sync HTTP), Claude claude-haiku-4-5-20251001

**Scope note:** 이 플랜은 Plan A (노트→AI→칸반 루프). 시간 측정·Workday·Report는 Plan B로 분리.

**Prerequisites:**
- `ANTHROPIC_API_KEY` 환경 변수 설정 필요
- `cargo tauri dev` 실행 가능 상태

---

## File Structure Map

| 파일 | 작업 | 역할 |
|------|------|------|
| `src-tauri/crates/core-shared/src/lib.rs` | 수정 | Task, TaskStatus, NoteSnapshot, ParsedChange, TaskGraphCommand, TimelineMemo 타입 |
| `src-tauri/crates/app-service/Cargo.toml` | 수정 | rusqlite, serde_json 의존성 추가 |
| `src-tauri/crates/app-service/src/db.rs` | 신규 | SQLite 스키마 생성 + CRUD 함수 |
| `src-tauri/crates/app-service/src/lib.rs` | 수정 | AppService 구체 구현 (제네릭 제거, db 필드 추가) |
| `src-tauri/crates/processing/Cargo.toml` | 수정 | ureq, serde, serde_json 의존성 추가 |
| `src-tauri/crates/processing/src/ai/mod.rs` | 수정 | parse_diff(), map_tasks() — Claude API 호출 |
| `src-tauri/Cargo.toml` | 수정 | workspace에 rusqlite 추가 |
| `src-tauri/src/lib.rs` | 수정 | AppService 상태 등록, Tauri commands 3개 추가 |
| `src/components/WorkView.tsx` | 수정 | 15s useEffect → invoke("save_snapshot") |
| `src/components/TaskKanban.tsx` | 신규 | task graph 기반 3컬럼 칸반 |
| `src/app-view/App.tsx` | 수정 | TaskKanban 마운트, task-graph-updated 이벤트 수신 |

---

## Task 1: Core Data Types

**Files:**
- Modify: `src-tauri/crates/core-shared/src/lib.rs`

- [ ] **Step 1: core-shared/src/lib.rs 전체 교체**

```rust
//! 공통 데이터 모델. 다른 크레이트가 의존하는 최하위 공유 레이어.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "Todo"),
            TaskStatus::InProgress => write!(f, "InProgress"),
            TaskStatus::Done => write!(f, "Done"),
        }
    }
}

impl TryFrom<&str> for TaskStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "Todo" => Ok(TaskStatus::Todo),
            "InProgress" => Ok(TaskStatus::InProgress),
            "Done" => Ok(TaskStatus::Done),
            other => Err(format!("unknown status: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub label: String,
    pub status: TaskStatus,
    pub parent_id: Option<i64>,
    pub source_text: String,
    pub cumulative_ms: i64,
    pub created_at: i64, // Unix timestamp (secs)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineMemo {
    pub id: i64,
    pub task_id: i64,
    pub event_type: String, // "NewTask" | "Expansion" | "Decomposition" | "StatusChange"
    pub description: String,
    pub timestamp: i64, // Unix timestamp (secs)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSnapshot {
    pub id: i64,
    pub captured_at: i64,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedChange {
    pub change_type: String, // "Added" | "Removed" | "Modified"
    pub text: String,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskGraphCommand {
    CreateTask {
        label: String,
        parent_id: Option<i64>,
        source_text: String,
    },
    UpdateTask {
        id: i64,
        status: Option<String>,
    },
    AddMemo {
        task_id: i64,
        event_type: String,
        description: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_roundtrip() {
        let s = TaskStatus::InProgress.to_string();
        let back = TaskStatus::try_from(s.as_str()).unwrap();
        assert_eq!(back, TaskStatus::InProgress);
    }

    #[test]
    fn task_status_unknown_returns_err() {
        assert!(TaskStatus::try_from("Bogus").is_err());
    }
}
```

- [ ] **Step 2: 컴파일 확인**

```bash
cd src-tauri && cargo check -p core-shared
```

Expected: `Finished` (warning 없거나 unused import 정도)

- [ ] **Step 3: 테스트 실행**

```bash
cd src-tauri && cargo test -p core-shared
```

Expected: `test task_status_roundtrip ... ok` `test task_status_unknown_returns_err ... ok`

- [ ] **Step 4: 커밋**

```bash
git add src-tauri/crates/core-shared/src/lib.rs
git commit -m "feat(core-shared): Task, NoteSnapshot, ParsedChange, TaskGraphCommand 타입 정의"
```

---

## Task 2: SQLite 의존성 + DB 스키마

**Files:**
- Modify: `src-tauri/Cargo.toml` (workspace deps)
- Modify: `src-tauri/crates/app-service/Cargo.toml`
- Create: `src-tauri/crates/app-service/src/db.rs`

- [ ] **Step 1: workspace Cargo.toml에 rusqlite 추가**

`src-tauri/Cargo.toml` 의 `[workspace.dependencies]` 섹션에 추가:

```toml
rusqlite = { version = "0.31", features = ["bundled"] }
```

- [ ] **Step 2: app-service Cargo.toml 업데이트**

`src-tauri/crates/app-service/Cargo.toml`:

```toml
[package]
name = "app-service"
version = "0.1.0"
edition = "2021"

[dependencies]
core-shared = { path = "../core-shared" }
rusqlite = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

- [ ] **Step 3: db.rs 작성**

Create `src-tauri/crates/app-service/src/db.rs`:

```rust
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
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id    INTEGER NOT NULL REFERENCES tasks(id),
            event_type TEXT NOT NULL,
            description TEXT NOT NULL,
            timestamp  INTEGER NOT NULL
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
```

- [ ] **Step 4: 컴파일 + 테스트**

```bash
cd src-tauri && cargo test -p app-service
```

Expected: 4 tests pass

- [ ] **Step 5: 커밋**

```bash
git add src-tauri/Cargo.toml src-tauri/crates/app-service/
git commit -m "feat(app-service): SQLite 스키마 + CRUD db.rs"
```

---

## Task 3: AppService 구체 구현

**Files:**
- Modify: `src-tauri/crates/app-service/src/lib.rs`

- [ ] **Step 1: lib.rs 교체**

```rust
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
```

- [ ] **Step 2: 컴파일 확인**

```bash
cd src-tauri && cargo check -p app-service
```

Expected: `Finished`

- [ ] **Step 3: 커밋**

```bash
git add src-tauri/crates/app-service/src/lib.rs
git commit -m "feat(app-service): AppService 구체 구현 (제네릭 제거, SQLite 통합)"
```

---

## Task 4: AI 에이전트 함수 (parse_diff + map_tasks)

**Files:**
- Modify: `src-tauri/crates/processing/Cargo.toml`
- Modify: `src-tauri/crates/processing/src/ai/mod.rs`

- [ ] **Step 1: processing Cargo.toml 업데이트**

```toml
[package]
name = "processing"
version = "0.1.0"
edition = "2021"

[dependencies]
core-shared = { path = "../core-shared" }
serde = { workspace = true }
serde_json = { workspace = true }
ureq = { version = "2", features = ["json"] }
```

- [ ] **Step 2: ai/mod.rs 작성**

```rust
//! Claude API 기반 AI 에이전트 함수.
//!
//! Agent 1 (parse_diff): 마크다운 diff → ParsedChange[]
//! Agent 2 (map_tasks): ParsedChange[] → TaskGraphCommand[]

use core_shared::{ParsedChange, Task, TaskGraphCommand};
use serde_json::{json, Value};

const MODEL: &str = "claude-haiku-4-5-20251001";
const API_URL: &str = "https://api.anthropic.com/v1/messages";

fn api_key() -> Result<String, String> {
    std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY 환경 변수가 설정되지 않았습니다".to_string())
}

fn call_claude(system: &str, user: &str) -> Result<String, String> {
    let key = api_key()?;
    let body = json!({
        "model": MODEL,
        "max_tokens": 1024,
        "system": system,
        "messages": [{"role": "user", "content": user}]
    });

    let response: Value = ureq::post(API_URL)
        .set("x-api-key", &key)
        .set("anthropic-version", "2023-06-01")
        .set("content-type", "application/json")
        .send_json(body)
        .map_err(|e| format!("API 호출 실패: {}", e))?
        .into_json()
        .map_err(|e| format!("응답 파싱 실패: {}", e))?;

    response["content"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "응답 text 필드 없음".to_string())
}

/// Agent 1: 마크다운 before/after diff → ParsedChange 목록
pub fn parse_diff(before: &str, after: &str) -> Result<Vec<ParsedChange>, String> {
    // 의미 없는 diff는 AI 호출 생략
    if before.trim() == after.trim() {
        return Ok(vec![]);
    }

    let system = "You are a markdown diff parser. Given before and after markdown text, identify meaningful changes (ignore whitespace-only changes). Return ONLY a valid JSON array with no extra text. Each element: {\"change_type\": \"Added\"|\"Removed\"|\"Modified\", \"text\": \"<the changed text>\", \"context\": \"<up to 3 surrounding lines for context>\"}";

    let user = format!(
        "BEFORE:\n{}\n\nAFTER:\n{}\n\nReturn JSON array of changes:",
        before, after
    );

    let raw = call_claude(system, &user)?;

    // JSON 배열 추출 (응답에 여분 텍스트가 있을 경우 대비)
    let start = raw.find('[').ok_or("응답에 JSON 배열 없음")?;
    let end = raw.rfind(']').ok_or("응답에 JSON 배열 끝 없음")? + 1;
    let json_str = &raw[start..end];

    serde_json::from_str(json_str)
        .map_err(|e| format!("ParsedChange 파싱 실패: {} — raw: {}", e, &raw[..raw.len().min(200)]))
}

/// Agent 2: ParsedChange 목록 + 기존 task graph → TaskGraphCommand 목록
pub fn map_tasks(changes: &[ParsedChange], existing_tasks: &[Task]) -> Result<Vec<TaskGraphCommand>, String> {
    if changes.is_empty() {
        return Ok(vec![]);
    }

    let tasks_json = serde_json::to_string(existing_tasks)
        .map_err(|e| format!("task 직렬화 실패: {}", e))?;
    let changes_json = serde_json::to_string(changes)
        .map_err(|e| format!("changes 직렬화 실패: {}", e))?;

    let system = "You are a task graph manager. Given text changes from a markdown note and existing tasks, produce task graph commands. Rules: 1) Map changes to existing tasks by text similarity first (threshold: 70%). 2) Indented items (2+ spaces or tab prefix in text/context) should have parent_id set to the closest parent task id. 3) Checkbox state changes [x] or [ ] produce UpdateTask with status Done or Todo. 4) New unmatched items produce CreateTask. Return ONLY a valid JSON array with no extra text. Commands: {\"type\":\"CreateTask\",\"label\":\"...\",\"parent_id\":null_or_number,\"source_text\":\"...\"} | {\"type\":\"UpdateTask\",\"id\":number,\"status\":\"Todo\"|\"InProgress\"|\"Done\"} | {\"type\":\"AddMemo\",\"task_id\":number,\"event_type\":\"Expansion\"|\"Decomposition\"|\"StatusChange\"|\"NewTask\",\"description\":\"...\"}";

    let user = format!(
        "EXISTING TASKS: {}\n\nCHANGES: {}\n\nReturn JSON array of TaskGraphCommands:",
        tasks_json, changes_json
    );

    let raw = call_claude(system, &user)?;

    let start = raw.find('[').ok_or("응답에 JSON 배열 없음")?;
    let end = raw.rfind(']').ok_or("응답에 JSON 배열 끝 없음")? + 1;
    let json_str = &raw[start..end];

    serde_json::from_str(json_str)
        .map_err(|e| format!("TaskGraphCommand 파싱 실패: {} — raw: {}", e, &raw[..raw.len().min(200)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_text_returns_empty_changes() {
        let result = parse_diff("same content", "same content").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn empty_changes_returns_empty_commands() {
        let result = map_tasks(&[], &[]).unwrap();
        assert!(result.is_empty());
    }
}
```

- [ ] **Step 3: 컴파일 + 단위 테스트 (API 호출 없는 것만)**

```bash
cd src-tauri && cargo test -p processing
```

Expected: `identical_text_returns_empty_changes ... ok`, `empty_changes_returns_empty_commands ... ok`

- [ ] **Step 4: 커밋**

```bash
git add src-tauri/crates/processing/
git commit -m "feat(processing): parse_diff, map_tasks — Claude API 에이전트 함수"
```

---

## Task 5: Tauri Commands 연결

**Files:**
- Modify: `src-tauri/Cargo.toml` (bin dependencies)
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: src-tauri/Cargo.toml의 [dependencies]에 추가**

기존 `[dependencies]` 섹션에 추가:
```toml
rusqlite = { workspace = true }
```

- [ ] **Step 2: src-tauri/src/lib.rs에 새 커맨드 + 상태 추가**

기존 CheckInState/CheckInData/관련 함수는 유지. 파일 상단에 추가:

```rust
use app_service::AppService;
```

파일 하단 `run()` 함수 내, `tauri::Builder::default()` 앞에 추가:

```rust
let svc = AppService::open("whatchadoin.db").expect("DB 초기화 실패");
```

`.manage(Mutex::new(CheckInState::default()))` 다음 줄에:
```rust
.manage(svc)
```

`.invoke_handler(...)` 안에 3개 커맨드 추가:
```rust
.invoke_handler(tauri::generate_handler![
    open_checkin,
    get_checkin_data,
    submit_checkin,
    save_snapshot,
    get_tasks,
    update_task_status_cmd,
])
```

- [ ] **Step 3: 커맨드 함수 3개 추가** (`submit_checkin` 함수 뒤에 삽입)

```rust
#[tauri::command]
fn save_snapshot(
    app: AppHandle,
    svc: tauri::State<'_, AppService>,
    content: String,
) -> Result<(), String> {
    let prev_opt = svc.save_snapshot(&content).map_err(|e| e.to_string())?;

    // diff 가 있을 때만 AI 파이프라인 실행
    let before = prev_opt.unwrap_or_default();
    let changes = processing::ai::parse_diff(&before, &content)
        .unwrap_or_else(|e| {
            eprintln!("[AI] parse_diff 실패: {}", e);
            vec![]
        });

    if !changes.is_empty() {
        let existing = svc.get_tasks().map_err(|e| e.to_string())?;
        let cmds = processing::ai::map_tasks(&changes, &existing)
            .unwrap_or_else(|e| {
                eprintln!("[AI] map_tasks 실패: {}", e);
                vec![]
            });

        if !cmds.is_empty() {
            svc.apply_commands(&cmds).map_err(|e| e.to_string())?;
            // 프론트엔드에 task graph 갱신 알림
            app.emit("task-graph-updated", ()).ok();
        }
    }

    Ok(())
}

#[tauri::command]
fn get_tasks(svc: tauri::State<'_, AppService>) -> Result<Vec<core_shared::Task>, String> {
    svc.get_tasks().map_err(|e| e.to_string())
}

#[tauri::command]
fn update_task_status_cmd(
    app: AppHandle,
    svc: tauri::State<'_, AppService>,
    task_id: i64,
    status: String,
) -> Result<(), String> {
    svc.update_task_status(task_id, &status)
        .map_err(|e| e.to_string())?;
    app.emit("task-graph-updated", ()).ok();
    Ok(())
}
```

- [ ] **Step 4: 빌드 확인**

```bash
cd src-tauri && cargo check
```

Expected: `Finished` (경고 가능, 에러 없어야 함)

- [ ] **Step 5: 커밋**

```bash
git add src-tauri/Cargo.toml src-tauri/src/lib.rs
git commit -m "feat(tauri): save_snapshot, get_tasks, update_task_status_cmd 커맨드 등록"
```

---

## Task 6: WorkView 15s 스냅샷 (Frontend)

**Files:**
- Modify: `src/components/WorkView.tsx`

- [ ] **Step 1: WorkView.tsx 교체**

```tsx
import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

const SAMPLE_NOTE = `# 오늘의 작업

## 진행 중
- [ ] 칸반 리포트 레이아웃 구현
- [ ] tracking 이벤트 스키마 정의

## 완료
- [x] Tauri 개발 환경 셋업
- [x] mockup 브랜치 생성

## 메모
화면만 띄워두지 말고 실제로 밀도있게...
`;

const SNAPSHOT_INTERVAL_MS = 15_000;

interface Props {
  note: string;
  onNoteChange: (note: string) => void;
}

function WorkView({ note, onNoteChange }: Props) {
  const noteRef = useRef(note);
  noteRef.current = note;

  useEffect(() => {
    const id = setInterval(() => {
      invoke("save_snapshot", { content: noteRef.current }).catch((e) =>
        console.error("[WorkView] save_snapshot 실패:", e)
      );
    }, SNAPSHOT_INTERVAL_MS);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="work-view">
      <div className="editor-toolbar">
        <span className="editor-title">Note</span>
        <span className="tracking-status">
          <span className="tracking-dot" />
          15s마다 변경 추적 중
        </span>
      </div>

      <textarea
        className="markdown-editor"
        value={note}
        onChange={(e) => onNoteChange(e.target.value)}
        spellCheck={false}
      />

      <div className="editor-footer">
        <span className="hint">자유롭게 작성하세요 — AI가 자동으로 분석합니다</span>
      </div>
    </div>
  );
}

export { SAMPLE_NOTE };
export default WorkView;
```

- [ ] **Step 2: 타입 에러 확인**

`tsc --noEmit` 로 타입 체크. WorkView의 Props 타입이 변경되었으므로 App.tsx에서 에러가 발생함. App.tsx 전체 교체는 Task 8에서 진행.

```bash
cd /Users/user/Documents/whatchadoin && npx tsc --noEmit 2>&1 | head -20
```

Expected: WorkView Props 관련 에러 보임 (Task 8에서 해결)

- [ ] **Step 3: 커밋**

```bash
git add src/components/WorkView.tsx
git commit -m "feat(frontend): WorkView — 15s 스냅샷 invoke, note/onNoteChange Props로 변경"
```

---

## Task 7: TaskKanban 컴포넌트

**Files:**
- Create: `src/components/TaskKanban.tsx`

- [ ] **Step 1: TaskKanban.tsx 작성**

```tsx
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface Task {
  id: number;
  label: string;
  status: "Todo" | "InProgress" | "Done";
  parent_id: number | null;
  source_text: string;
  cumulative_ms: number;
  created_at: number;
}

const COLUMNS: { key: Task["status"]; label: string }[] = [
  { key: "Todo", label: "해야할 일" },
  { key: "InProgress", label: "진행 중" },
  { key: "Done", label: "완료" },
];

function TaskKanban() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);

  async function fetchTasks() {
    try {
      const result = await invoke<Task[]>("get_tasks");
      setTasks(result);
    } catch (e) {
      console.error("[TaskKanban] get_tasks 실패:", e);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    fetchTasks();
    const unlisten = listen("task-graph-updated", () => fetchTasks());
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  async function handleStatusChange(taskId: number, newStatus: Task["status"]) {
    try {
      await invoke("update_task_status_cmd", {
        taskId,
        status: newStatus,
      });
    } catch (e) {
      console.error("[TaskKanban] update_task_status_cmd 실패:", e);
    }
  }

  if (loading) return <div className="kanban-loading">칸반 로딩 중...</div>;

  if (tasks.length === 0) {
    return (
      <div className="kanban-empty">
        <p>노트에 할 일을 작성하면 AI가 자동으로 여기에 표시합니다.</p>
        <p style={{ fontSize: "12px", color: "#888" }}>예: <code>- [ ] 기능 구현</code></p>
      </div>
    );
  }

  return (
    <div className="kanban-board">
      {COLUMNS.map((col) => {
        const colTasks = tasks.filter(
          (t) => t.status === col.key && t.parent_id === null
        );
        return (
          <div key={col.key} className="kanban-column">
            <h4 className="kanban-col-title">
              {col.label}
              <span className="kanban-count">{colTasks.length}</span>
            </h4>
            {colTasks.map((task) => {
              const subtasks = tasks.filter((t) => t.parent_id === task.id);
              return (
                <div key={task.id} className="kanban-card">
                  <p className="kanban-label">{task.label}</p>
                  {subtasks.length > 0 && (
                    <ul className="kanban-subtasks">
                      {subtasks.map((sub) => (
                        <li key={sub.id} className={sub.status === "Done" ? "done" : ""}>
                          {sub.label}
                        </li>
                      ))}
                    </ul>
                  )}
                  <div className="kanban-actions">
                    {COLUMNS.filter((c) => c.key !== col.key).map((c) => (
                      <button
                        key={c.key}
                        className="kanban-move-btn"
                        onClick={() => handleStatusChange(task.id, c.key)}
                      >
                        → {c.label}
                      </button>
                    ))}
                  </div>
                </div>
              );
            })}
          </div>
        );
      })}
    </div>
  );
}

export default TaskKanban;
```

- [ ] **Step 2: 기본 스타일 추가** (`src/app-view/App.css` 하단에 추가)

```css
/* Kanban */
.kanban-board { display: flex; gap: 16px; padding: 16px; height: 100%; overflow-x: auto; }
.kanban-column { flex: 1; min-width: 200px; background: #f5f5f5; border-radius: 8px; padding: 12px; }
.kanban-col-title { font-size: 13px; font-weight: 700; margin: 0 0 10px; display: flex; justify-content: space-between; }
.kanban-count { background: #ddd; border-radius: 99px; padding: 1px 7px; font-size: 11px; }
.kanban-card { background: #fff; border: 1px solid #e0e0e0; border-radius: 6px; padding: 10px; margin-bottom: 8px; }
.kanban-label { margin: 0 0 6px; font-size: 13px; font-weight: 600; }
.kanban-subtasks { margin: 4px 0 8px 16px; padding: 0; list-style: disc; font-size: 12px; color: #555; }
.kanban-subtasks .done { text-decoration: line-through; color: #aaa; }
.kanban-actions { display: flex; gap: 6px; flex-wrap: wrap; }
.kanban-move-btn { font-size: 11px; padding: 2px 8px; border: 1px solid #ccc; border-radius: 4px; background: #fafafa; cursor: pointer; }
.kanban-move-btn:hover { background: #eee; }
.kanban-empty { padding: 32px; text-align: center; color: #888; font-size: 14px; }
.kanban-loading { padding: 16px; color: #888; font-size: 13px; }
```

- [ ] **Step 3: 커밋**

```bash
git add src/components/TaskKanban.tsx src/app-view/App.css
git commit -m "feat(frontend): TaskKanban 컴포넌트 — task graph 기반 3컬럼 칸반"
```

---

## Task 8: App.tsx 통합

**Files:**
- Modify: `src/app-view/App.tsx`

- [ ] **Step 1: App.tsx 전체 교체**

```tsx
import { useState, useEffect } from "react";
import LeftSidebar from "../components/LeftSidebar";
import RightSidebar from "../components/RightSidebar";
import WorkView, { SAMPLE_NOTE } from "../components/WorkView";
import ReportView from "../components/ReportView";
import TaskKanban from "../components/TaskKanban";
import "./App.css";

type Tab = "work" | "report";
const CHECK_IN_INTERVAL_MS = 60_000;

function App() {
  const [tab, setTab] = useState<Tab>("work");
  const [note, setNote] = useState(SAMPLE_NOTE);

  // Check-in 팝업 주기 (기존 로직 유지)
  useEffect(() => {
    const id = setInterval(() => {
      // TODO: open_checkin invoke (Plan B에서 구현)
    }, CHECK_IN_INTERVAL_MS);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="app-layout">
      <LeftSidebar currentTab={tab} onTabChange={setTab} />

      <main className="main-content">
        {tab === "work" ? (
          <div className="work-layout">
            <WorkView note={note} onNoteChange={setNote} />
            <TaskKanban />
          </div>
        ) : (
          <ReportView />
        )}
      </main>

      <RightSidebar events={[]} />
    </div>
  );
}

export default App;
```

- [ ] **Step 2: work-layout 스타일 추가** (`src/app-view/App.css` 하단에 추가)

```css
.work-layout { display: flex; flex-direction: column; height: 100%; overflow: hidden; }
.work-layout > .work-view { flex: 0 0 45%; border-bottom: 1px solid #e0e0e0; }
.work-layout > .kanban-board { flex: 1; }
```

- [ ] **Step 3: RightSidebar Props 확인**

`src/components/RightSidebar.tsx`를 열어 `events` prop 타입 확인. 기존 `TrackedEvent[]` 타입 그대로 전달 가능하면 OK. 타입 에러 나면:

```tsx
<RightSidebar events={[]} />
```

이 부분에서 타입 에러 나면 RightSidebar의 Props 타입을 확인해 맞게 수정.

- [ ] **Step 4: `cargo tauri dev`로 전체 통합 확인**

```bash
npm run tauri dev
```

확인 사항:
1. Work 탭에서 노트 에디터 + 하단 칸반 영역 보임
2. 칸반 "노트에 할 일을 작성하면..." 빈 상태 메시지 보임
3. `ANTHROPIC_API_KEY` 설정 후: 노트에 `- [ ] 테스트 태스크` 작성 → 15초 기다림 → 칸반에 카드 나타남
4. 칸반 카드의 "→ 진행 중" 버튼 클릭 → 상태 변경됨

- [ ] **Step 5: 커밋**

```bash
git add src/app-view/App.tsx src/app-view/App.css
git commit -m "feat(frontend): App.tsx에 WorkView + TaskKanban 통합 (work-layout)"
```

---

## 완료 기준

- [ ] `ANTHROPIC_API_KEY` 없을 때 앱 크래시 없이 실행됨 (에러 로그만)
- [ ] 노트 편집 → 15s → AI 처리 → 칸반 카드 자동 생성
- [ ] 칸반 카드 상태 버튼으로 Todo/InProgress/Done 전환 가능
- [ ] 앱 재시작 후 task graph 유지됨 (SQLite 영속)
- [ ] `cargo test -p core-shared -p app-service -p processing` 전체 통과

---

## 참고: Plan B 범위 (별도 플랜)

- Focus/Background 시간 측정 (F2·F3·F4)
- Spotlight Check-in 팝업 개선 (F5·F6)
- Workday 시작/종료 (F7)
- 일과 종료 리포트 (F9)
- Right Sidebar PatternEvent 스트림 (F15)
