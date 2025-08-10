use crate::debug_log;
use crate::util::FilePreview;
use base64::Engine as _;
use tauri_plugin_android_fs::{AndroidFsExt, FileUri, ImageFormat, Size};
use uuid::Uuid;

#[cfg(target_os = "android")]
#[tauri::command]
pub async fn get_file_infos_with_previews(
    app: tauri::AppHandle,
) -> Result<Vec<FilePreview>, String> {
    use tauri_plugin_android_fs::{AndroidFsExt, ImageFormat as FsImageFormat, Size};

    let app = app.clone(); // clone app handle to own it

    let uris = {
        let app = app.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let fs = app.android_fs();
            fs.show_open_file_dialog(None, &["*/*"], true)
        })
        .await
        .map_err(|e| format!("Task join error: {e}"))?
        .map_err(|e| format!("FS error: {e}"))?
    };

    let mut previews = vec![];

    for uri in uris {
        debug_log!("[DEBUG][Android FileUri]\n{:#?}", uri);

        let name = {
            let app = app.clone();
            let uri = uri.clone();
            tauri::async_runtime::spawn_blocking(move || {
                let fs = app.android_fs();
                fs.get_name(&uri)
            })
            .await
            .map_err(|e| format!("Task join error: {e}"))?
            .map_err(|e| format!("FS error: {e}"))?
        };

        let mime = {
            let app = app.clone();
            let uri = uri.clone();
            tauri::async_runtime::spawn_blocking(move || {
                let fs = app.android_fs();
                fs.get_mime_type(&uri)
            })
            .await
            .map_err(|e| format!("Task join error: {e}"))?
            .map_err(|e| format!("FS error: {e}"))?
            .unwrap_or_default()
        };

        let bytes = {
            let app = app.clone();
            let uri = uri.clone();
            tauri::async_runtime::spawn_blocking(move || {
                let fs = app.android_fs();
                fs.read(&uri)
            })
            .await
            .map_err(|e| format!("Task join error: {e}"))?
            .map_err(|e| format!("FS error: {e}"))?
        };

        let size = bytes.len() as u64;

        let preview_base64 = if mime.starts_with("image/") {
            let app = app.clone();
            let uri = uri.clone();
            tauri::async_runtime::spawn_blocking(move || {
                let fs = app.android_fs();
                match fs.get_thumbnail(
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
            .map_err(|e| format!("Task join error: {e}"))?
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
