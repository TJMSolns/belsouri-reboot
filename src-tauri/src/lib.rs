pub mod app_state;
pub mod db;
pub mod events;
pub mod licensing;
pub mod practice_setup;
pub mod projections;

use std::sync::Mutex;
use tauri::Manager;
use app_state::AppState;
use licensing::commands::{activate_license, get_license_status, get_practice_id, startup_license_check};
use practice_setup::commands::{
    update_practice_details, get_practice,
    create_office, rename_office, update_office_chair_count,
    set_office_hours, close_office_day, archive_office, list_offices, get_office,
    register_provider, rename_provider, change_provider_type,
    assign_provider_to_office, remove_provider_from_office,
    set_provider_availability, clear_provider_availability,
    set_provider_exception, remove_provider_exception,
    archive_provider, unarchive_provider, list_providers, get_provider,
    define_procedure_type, update_procedure_type,
    deactivate_procedure_type, reactivate_procedure_type,
    seed_default_procedure_types, list_procedure_types,
};

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
                update_practice_details, get_practice,
                create_office, rename_office, update_office_chair_count,
                set_office_hours, close_office_day, archive_office, list_offices, get_office,
                register_provider, rename_provider, change_provider_type,
                assign_provider_to_office, remove_provider_from_office,
                set_provider_availability, clear_provider_availability,
                set_provider_exception, remove_provider_exception,
                archive_provider, unarchive_provider, list_providers, get_provider,
                define_procedure_type, update_procedure_type,
                deactivate_procedure_type, reactivate_procedure_type,
                seed_default_procedure_types, list_procedure_types,
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
            update_practice_details, get_practice,
            create_office, rename_office, update_office_chair_count,
            set_office_hours, close_office_day, archive_office, list_offices, get_office,
            register_provider, rename_provider, change_provider_type,
            assign_provider_to_office, remove_provider_from_office,
            set_provider_availability, clear_provider_availability,
            set_provider_exception, remove_provider_exception,
            archive_provider, unarchive_provider, list_providers, get_provider,
            define_procedure_type, update_procedure_type,
            deactivate_procedure_type, reactivate_procedure_type,
            seed_default_procedure_types, list_procedure_types,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
