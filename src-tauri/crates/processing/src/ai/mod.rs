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
    if before.trim() == after.trim() {
        return Ok(vec![]);
    }

    let system = "You are a markdown diff parser. Given before and after markdown text, identify meaningful changes (ignore whitespace-only changes). Return ONLY a valid JSON array with no extra text. Each element: {\"change_type\": \"Added\"|\"Removed\"|\"Modified\", \"text\": \"<the changed text>\", \"context\": \"<up to 3 surrounding lines for context>\"}";

    let user = format!(
        "BEFORE:\n{}\n\nAFTER:\n{}\n\nReturn JSON array of changes:",
        before, after
    );

    let raw = call_claude(system, &user)?;

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
