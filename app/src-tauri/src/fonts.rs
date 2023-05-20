use font_kit::source::SystemSource;

pub struct Font {
    pub path: String,
}

#[tauri::command]
pub fn get_system_fonts() -> Option<Vec<String>> {
    let source = SystemSource::new();

    let found_families = source.all_families();

    found_families.ok()
}
