use crate::debug_log;
use crate::APP_HANDLE;
use bytes::Buf;
use futures_util::{StreamExt, TryStreamExt};
use rcgen::generate_simple_self_signed;
use sanitize_filename::sanitize;
use tauri_plugin_android_fs::{AndroidFsExt, PrivateDir, PublicGeneralPurposeDir};
use warp::{self, http::StatusCode, multipart::FormData, Rejection, Reply};

// Find the downloads directory path and save files in the downloads directory
pub async fn save_file(mut form: FormData) -> Result<impl Reply, Rejection> {
    let app = APP_HANDLE.get().ok_or_else(|| {
        debug_log!("App handle not initialized");
        warp::reject::custom(super::InternalServerError)
    })?;

    let api = app.android_fs();
    let storage = api.public_storage();

    while let Ok(Some(part)) = form.try_next().await {
        if let Some(filename) = part.filename() {
            let safe_filename = sanitize(filename);
            let mime_type = part
                .content_type()
                .map(|ct| ct.to_string())
                .unwrap_or("application/octet-stream".into());

            // Create file in Android public Downloads folder
            let uri = storage
                .create_file(
                    PublicGeneralPurposeDir::Download,
                    &safe_filename,
                    Some(&mime_type),
                )
                .map_err(|e| {
                    debug_log!("Failed to create MediaStore file: {:?}", e);
                    warp::reject::custom(super::InternalServerError)
                })?;

            // Read the file data from the stream
            let mut file_data = Vec::new();
            let mut part_stream = part.stream();
            while let Some(chunk_result) = part_stream.next().await {
                let mut chunk = chunk_result.map_err(|e| {
                    debug_log!("Stream read failed: {}", e);
                    warp::reject::custom(super::InternalServerError)
                })?;

                while chunk.has_remaining() {
                    let bytes = chunk.chunk();
                    file_data.extend_from_slice(bytes);
                    let len = bytes.len();
                    chunk.advance(len);
                }
            }

            // Write data to the file via plugin API
            api.write(&uri, &file_data).map_err(|e| {
                debug_log!("Failed to write file via android-fs: {:?}", e);
                warp::reject::custom(super::InternalServerError)
            })?;
        }
    }

    Ok(warp::reply::with_status(
        "File(s) uploaded successfully.",
        StatusCode::CREATED,
    ))
}

// Generate TLS certificates required for warp https server
// and save them to apps private cache directory
pub fn generate_tls_certs_to_disk(
    app: &tauri::AppHandle,
) -> Result<(std::path::PathBuf, std::path::PathBuf), Box<dyn std::error::Error>> {
    // Generate the cert
    let subject_alt_names = vec!["localhost".to_string()];
    let cert_key = generate_simple_self_signed(subject_alt_names)?;

    let cert_pem = cert_key.cert.pem();
    let key_pem = cert_key.signing_key.serialize_pem();

    // Use Tauri plugin FS to get app-private cache dir
    let fs = app.android_fs();
    let base_dir = fs.private_storage().resolve_path(PrivateDir::Cache)?;
    let certs_dir = base_dir.join("certs");

    std::fs::create_dir_all(&certs_dir)?;

    let cert_path = certs_dir.join("cert.pem");
    let key_path = certs_dir.join("key.pem");

    std::fs::write(&cert_path, cert_pem)?;
    std::fs::write(&key_path, key_pem)?;

    debug_log!("📱 [Android] TLS cert written to: {}", cert_path.display());
    debug_log!("📱 [Android] TLS key written to: {}", key_path.display());

    Ok((cert_path, key_path))
}
