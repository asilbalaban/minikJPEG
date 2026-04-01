// Tauri Windows'ta konsol penceresi açmasın (release modunda)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    minikjpeg_tauri_lib::run();
}
