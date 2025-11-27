mod lowres;
use lowres::LowresConfig;
use std::path::PathBuf;

use base64::Engine;
use std::fs::File;
use std::io::Read;

fn file_to_base64(path: &PathBuf) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    // Determine mime type based on extension
    let ext = path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    };

    let b64 = base64::engine::general_purpose::STANDARD.encode(buffer);
    Ok(format!("data:{};base64,{}", mime, b64))
}

#[tauri::command]
async fn get_image_base64(path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(path);
    file_to_base64(&path_buf)
}

#[tauri::command]
async fn process_image(input: String, config: LowresConfig) -> Result<(String, String), String> {
    let input_path = PathBuf::from(&input);
    let file_stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = input_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    let output_filename = format!("{}_lowres.png", file_stem);
    let output_path = parent.join(output_filename);

    lowres::process_image(input_path, output_path.clone(), config).map_err(|e| e.to_string())?;

    let b64 = file_to_base64(&output_path)?;
    Ok((output_path.to_string_lossy().to_string(), b64))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![process_image, get_image_base64])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
