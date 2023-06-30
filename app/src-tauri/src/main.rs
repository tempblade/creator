// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use creator_core::{
    __cmd__calculate_timeline_at_curr_frame, __cmd__get_system_families, __cmd__get_system_font,
    __cmd__get_system_fonts, __cmd__get_values_at_frame_range_from_animated_float,
    __cmd__get_values_at_frame_range_from_animated_float_vec2,
    __cmd__get_values_at_frame_range_from_animated_float_vec3,
    animation::{
        primitives::values::animated_values::{
            get_values_at_frame_range_from_animated_float,
            get_values_at_frame_range_from_animated_float_vec2,
            get_values_at_frame_range_from_animated_float_vec3,
        },
        timeline::calculate_timeline_at_curr_frame,
    },
    fonts::fonts::{get_system_families, get_system_font, get_system_fonts},
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            calculate_timeline_at_curr_frame,
            get_system_font,
            get_system_families,
            get_system_fonts,
            get_values_at_frame_range_from_animated_float,
            get_values_at_frame_range_from_animated_float_vec2,
            get_values_at_frame_range_from_animated_float_vec3
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
