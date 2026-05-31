use std::sync::Mutex;
use tauri::{AppHandle, Emitter, EventTarget, Manager, WebviewUrl, WebviewWindowBuilder};
use app_service::AppService;

#[derive(Default)]
struct CheckInState {
    tasks: Vec<String>,
    active_task: Option<String>,
}

#[derive(serde::Serialize, Clone)]
struct CheckInData {
    tasks: Vec<String>,
    active_task: Option<String>,
}

#[tauri::command]
fn open_checkin(
    app: AppHandle,
    state: tauri::State<'_, Mutex<CheckInState>>,
    tasks: Vec<String>,
    active_task: Option<String>,
) -> Result<(), String> {
    println!(
        "[Rust] open_checkin called, tasks: {:?}, active_task: {:?}",
        tasks, active_task
    );
    let checkin_data = CheckInData { tasks, active_task };
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.tasks = checkin_data.tasks.clone();
        s.active_task = checkin_data.active_task.clone();
    }

    if let Some(win) = app.get_webview_window("checkin") {
        println!("[Rust] checkin window exists, showing");
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        app.emit_to(
            EventTarget::WebviewWindow {
                label: "checkin".to_string(),
            },
            "checkin://data-updated",
            checkin_data,
        )
        .map_err(|e| e.to_string())?;
    } else {
        println!("[Rust] creating new checkin window");
        WebviewWindowBuilder::new(&app, "checkin", WebviewUrl::App(Default::default()))
            .title("지금 뭐 하고 있어?")
            .inner_size(360.0, 400.0)
            .center()
            .always_on_top(true)
            .resizable(false)
            .build()
            .map_err(|e| e.to_string())?;
        println!("[Rust] checkin window created");
    }

    Ok(())
}

#[tauri::command]
fn get_checkin_data(state: tauri::State<'_, Mutex<CheckInState>>) -> CheckInData {
    let s = state.lock().unwrap();
    println!(
        "[Rust] get_checkin_data called, tasks: {:?}, active_task: {:?}",
        s.tasks, s.active_task
    );
    CheckInData {
        tasks: s.tasks.clone(),
        active_task: s.active_task.clone(),
    }
}

#[tauri::command]
fn submit_checkin(app: AppHandle, task: String) -> Result<(), String> {
    println!("[Rust] submit_checkin called, task: {}", task);
    app.emit_to(
        EventTarget::WebviewWindow {
            label: "main".to_string(),
        },
        "checkin://submit",
        task,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_snapshot(
    app: AppHandle,
    svc: tauri::State<'_, AppService>,
    content: String,
) -> Result<(), String> {
    let prev_opt = svc.save_snapshot(&content).map_err(|e| e.to_string())?;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let svc = AppService::open("whatchadoin.db").expect("DB 초기화 실패");

    tauri::Builder::default()
        .manage(Mutex::new(CheckInState::default()))
        .manage(svc)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_checkin,
            get_checkin_data,
            submit_checkin,
            save_snapshot,
            get_tasks,
            update_task_status_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
