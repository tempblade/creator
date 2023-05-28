use font_kit::source::SystemSource;

#[tauri::command]
pub fn get_system_fonts() -> Option<Vec<String>> {
    let source = SystemSource::new();

    let found_fonts = source.all_fonts();

    match found_fonts {
        Ok(found_fonts) => {
            let font_names: Vec<String> = found_fonts
                .iter()
                .map(|f| f.load())
                .filter(|f| f.is_ok())
                .map(|f| f.unwrap())
                .map(|f| f.postscript_name())
                .filter(|f| f.is_some())
                .map(|f| f.unwrap())
                .collect();

            Some(font_names)
        }
        Err(_) => None,
    }
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
