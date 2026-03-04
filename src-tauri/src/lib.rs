pub mod app_state;
pub mod db;
pub mod events;
pub mod licensing;
pub mod projections;

use std::sync::Mutex;
use tauri::Manager;
use app_state::AppState;
use licensing::commands::{activate_license, get_license_status, get_practice_id, startup_license_check};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    {
        use specta_typescript::{BigIntExportBehavior, Typescript};
        use tauri_specta::{collect_commands, Builder};

        Builder::<tauri::Wry>::new()
            .commands(collect_commands![
                startup_license_check,
                get_license_status,
                get_practice_id,
                activate_license,
            ])
            .export(
                Typescript::default().bigint(BigIntExportBehavior::Number),
                "../src/lib/bindings.ts",
            )
            .expect("Failed to export TypeScript bindings");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_local_data_dir()
                .map_err(|e| format!("Could not resolve app data dir: {e}"))?;

            std::fs::create_dir_all(&app_dir)
                .map_err(|e| format!("Could not create app data dir: {e}"))?;

            let event_store = db::EventStore::open(&app_dir.join("events.db"))
                .map_err(|e| format!("Could not open event store: {e}"))?;
            let proj_store = db::ProjectionStore::open(&app_dir.join("projections.db"))
                .map_err(|e| format!("Could not open projection store: {e}"))?;

            app.manage(AppState {
                events: Mutex::new(event_store),
                projections: Mutex::new(proj_store),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            startup_license_check,
            get_license_status,
            get_practice_id,
            activate_license,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
