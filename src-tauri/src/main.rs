// Entry binary Tauri. Mencegah jendela konsol tambahan di Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    caption_tauri_lib::run();
}
