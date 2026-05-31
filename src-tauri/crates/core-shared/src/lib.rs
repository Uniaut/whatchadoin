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
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineMemo {
    pub id: i64,
    pub task_id: i64,
    pub event_type: String,
    pub description: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSnapshot {
    pub id: i64,
    pub captured_at: i64,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedChange {
    pub change_type: String,
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
