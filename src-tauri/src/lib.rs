//! Library `caption-tauri`: command yang menghubungkan UI ke `caption-core`.

pub mod commands;

/// Entry Tauri: daftarkan command & plugin, lalu jalankan aplikasi.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_models,
            commands::import_model,
            commands::pick_input_file,
            commands::validate_size_cmd,
            commands::list_presets,
            commands::render_ass_from_words,
            commands::transcribe,
        ])
        .run(tauri::generate_context!())
        .expect("gagal menjalankan Tauri");
}

#[cfg(test)]
mod commands_tests;
