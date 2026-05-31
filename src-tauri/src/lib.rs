use std::sync::Mutex;
use tauri::{AppHandle, Emitter, EventTarget, Manager, WebviewUrl, WebviewWindowBuilder};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 조립기(tauri-main): 처리 엔진을 생성해 app-service에 주입한다.
    let engine = processing::ProcessingEngine::default();
    let _app_service = app_service::AppService::new(engine);
    let _collector = collection::Collector::default();
    // TODO: _app_service를 tauri state로 등록, _collector.start()로 추적 시작.
    //   core-shared의 ReportDto는 향후 tauri command 반환 타입으로 사용.

    tauri::Builder::default()
        .manage(Mutex::new(CheckInState::default()))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_checkin,
            get_checkin_data,
            submit_checkin
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
