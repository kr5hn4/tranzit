use crate::debug_log;
use base64::{engine::general_purpose, Engine as _};
use if_addrs::get_if_addrs;
use image::{codecs::jpeg::JpegEncoder, DynamicImage, ExtendedColorType, ImageReader};
use infer;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::net::IpAddr;
use std::{fs, path::PathBuf};

// Get primary ipv4
// TODO: Get all local ips and do a mdns discovery on all local ips instead of just the first found
// address (should virtual interfaces be ignored? is there a reliable way to detect virtual
// interfaces?)
#[tauri::command]
pub fn get_primary_ipv4() -> Result<IpAddr, String> {
    let interfaces = get_if_addrs().map_err(|e| format!("Failed to get interfaces: {}", e))?;

    for iface in interfaces {
        if iface.is_loopback() {
            continue;
        }

        if let IpAddr::V4(ipv4) = iface.ip() {
            let octets = ipv4.octets();
            if octets[0] == 169 && octets[1] == 254 {
                continue; // Skip link-local IPs
            }

            debug_log!("✅ Chosen interface: {} | IP: {}", iface.name, ipv4);
            return Ok(IpAddr::V4(ipv4));
        }
    }

    Err("No suitable non-loopback IPv4 address found.".into())
}

#[derive(Serialize)]
pub struct FilePreview {
    pub file_uuid: String,
    pub file_path: String,
    pub name: String,
    pub size: u64,
    pub mime_type: String,
    pub preview_base64: Option<String>,
}

#[derive(Deserialize)]
pub struct FileInput {
    file_path: String,
    file_uuid: String,
}

// get file name, full path, size, and previews for image files
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
#[tauri::command]
pub async fn get_file_infos_with_previews(
    paths: Vec<FileInput>,
) -> Result<Vec<FilePreview>, String> {
    let mut result = Vec::new();

    for input in paths {
        let file_uuid = input.file_uuid.clone();
        let path_str = input.file_path;
        let path = PathBuf::from(&path_str);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let metadata = fs::metadata(&path)
            .map_err(|e| format!("Failed to get metadata for {}: {}", name, e))?;
        let size = metadata.len();

        let mime_type = infer::get_from_path(&path)
            .ok()
            .flatten()
            .map(|kind| kind.mime_type().to_string())
            .or_else(|| {
                mime_guess::from_path(&path)
                    .first()
                    .map(|m| m.essence_str().to_string())
            })
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let preview_base64 = if mime_type.starts_with("image/") {
            ImageReader::open(&path)
                .ok()
                .and_then(|reader| reader.decode().ok())
                .map(|image| generate_base64_thumbnail(&image))
                .transpose()?
        } else {
            None
        };

        result.push(FilePreview {
            file_path: path_str,
            name,
            size,
            mime_type,
            preview_base64,
            file_uuid,
        });
    }

    Ok(result)
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn generate_base64_thumbnail(image: &DynamicImage) -> Result<String, String> {
    let thumbnail = image.thumbnail(200, 200);
    let rgb_image = thumbnail.to_rgb8();
    let (width, height) = rgb_image.dimensions();

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    let mut encoder = JpegEncoder::new_with_quality(&mut cursor, 70);

    encoder
        .encode(&rgb_image, width, height, ExtendedColorType::Rgb8)
        .map_err(|e| format!("Failed to encode JPEG: {}", e))?;

    Ok(general_purpose::STANDARD.encode(&buffer))
}

// Enable devtools in debug builds
#[cfg(debug_assertions)]
pub fn prevent_default() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    use tauri_plugin_prevent_default::Flags;

    tauri_plugin_prevent_default::Builder::new()
        .with_flags(Flags::all().difference(Flags::DEV_TOOLS | Flags::RELOAD))
        .build()
}

//disable context menus, reload and dev tools in release builds
#[cfg(not(debug_assertions))]
pub fn prevent_default() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_prevent_default::init()
}
