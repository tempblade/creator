// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::{
    animation::timeline::calculate_timeline_entities_at_frame,
    fonts::{get_system_families, get_system_font, get_system_fonts},
};

pub mod animation;
pub mod fonts;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            calculate_timeline_entities_at_frame,
            get_system_font,
            get_system_families,
            get_system_fonts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
