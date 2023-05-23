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

#[tauri::command]
pub fn get_system_font(font_name: String) -> Option<Vec<u8>> {
    let source = SystemSource::new();

    let font = source.select_by_postscript_name(font_name.as_str());

    match font {
        Ok(font) => {
            let font = font.load();

            if let Ok(font) = font {
                if let Some(font_data) = font.copy_font_data() {
                    Some(font_data.as_slice().to_owned())
                } else {
                    None
                }
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
