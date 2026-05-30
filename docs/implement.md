my-tauri-app/
├── src/                        # Frontend Layer
│   ├── popup-view/
│   └── app-view/
│
└── src-tauri/                  # Rust Backend (Workspace Root)
    ├── Cargo.toml
    └── crates/
        ├── tauri-main/         # 조립기 (ProcessingEngine을 생성하여 App Service에 주입)
        │
        ├── app-service/        # UI 요청 처리 + ProcessingEngine을 호출하여 리포트/QA 데이터 획득
        │
        ├── collection/         # 백그라운드 추적 및 데이터 수집 파이프라인
        │
        ├── processing/         # 비즈니스 로직 핵심
        │   ├── src/
        │   │   ├── ai/         # AI API Call 로직
        │   │   ├── session/    # 세션 Segmentation 알고리즘
        │   │   └── api/        # App Service가 호출할 외부 노출 API (Trait 구현부)
        │
        └── core-shared/        # 공통 데이터 모델(ReportDto 등) 및 인터페이스(Trait) 정의