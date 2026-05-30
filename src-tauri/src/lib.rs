// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
