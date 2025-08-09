use crate::debug_log;
use tauri_plugin_android_fs::{AndroidFsExt, FileUri, ImageFormat, Size};
use uuid::Uuid;

// get file name, full path, size, and previews in base64 for image files
#[cfg(target_os = "android")]
#[tauri::command]
pub async fn get_file_infos_with_previews(
    app: tauri::AppHandle,
) -> Result<Vec<FilePreview>, String> {
    use tauri_plugin_android_fs::{AndroidFsExt, ImageFormat as FsImageFormat, Size};
    let fs = app.android_fs();

    // Show file dialog - if this is sync and blocks, run in spawn_blocking
    let uris = tauri::async_runtime::spawn_blocking(move || {
        fs.show_open_file_dialog(None, &["*/*"], true)
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    let mut previews = vec![];

    for uri in uris {
        debug_log!("[DEBUG][Android FileUri]\n{:#?}", uri);

        // Similarly wrap sync fs calls:
        let name = tauri::async_runtime::spawn_blocking({
            let fs = fs.clone();
            let uri = uri.clone();
            move || fs.get_name(&uri)
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))??;

        let mime = tauri::async_runtime::spawn_blocking({
            let fs = fs.clone();
            let uri = uri.clone();
            move || fs.get_mime_type(&uri)
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))??
        .unwrap_or_default();

        let bytes = tauri::async_runtime::spawn_blocking({
            let fs = fs.clone();
            let uri = uri.clone();
            move || fs.read(&uri)
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))??;

        let size = bytes.len() as u64;

        let preview_base64 = if mime.starts_with("image/") {
            tauri::async_runtime::spawn_blocking({
                let fs = fs.clone();
                let uri = uri.clone();
                move || match fs.get_thumbnail(
                    &uri,
                    Size {
                        width: 100,
                        height: 100,
                    },
                    FsImageFormat::Jpeg,
                ) {
                    Ok(Some(thumbnail_bytes)) => {
                        Some(base64::engine::general_purpose::STANDARD.encode(&thumbnail_bytes))
                    }
                    _ => None,
                }
            })
            .await
            .map_err(|e| format!("Task join error: {e}"))??
        } else {
            None
        };

        previews.push(FilePreview {
            file_path: uri.uri.clone(),
            file_uuid: uuid::Uuid::new_v4().to_string(),
            name,
            size,
            mime_type: mime,
            preview_base64,
        });
    }

    Ok(previews)
}
