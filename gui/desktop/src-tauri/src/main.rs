// Suffice GUI desktop shell. The window loads the same bundle gui/web ships;
// all data flows through the app-server WebSocket exactly as in the browser,
// so this crate stays a dumb frame with no model-visible surface at all.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("Suffice GUI penceresi başlatılamadı");
}
