//! Library `caption-tauri`: command yang menghubungkan UI ke `caption-core`.

pub mod commands;

/// Entry Tauri: daftarkan command & plugin, lalu jalankan aplikasi.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(commands::BurnManager::default())
        .manage(commands::TranscribeManager::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_models,
            commands::models_directory,
            commands::import_model,
            commands::pick_input_file,
            commands::validate_size_cmd,
            commands::list_presets,
            commands::render_ass_from_words,
            commands::write_export_file,
            commands::burn_video,
            commands::cancel_burn,
            commands::transcribe,
            commands::cancel_transcribe,
        ])
        .run(tauri::generate_context!())
        .expect("gagal menjalankan Tauri");
}

#[cfg(test)]
mod commands_tests;
