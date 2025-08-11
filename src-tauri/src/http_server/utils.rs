use crate::debug_log;
use bytes::Buf;
use dirs_next::download_dir;
use futures_util::{StreamExt, TryStreamExt};
use rcgen::generate_simple_self_signed;
use sanitize_filename::sanitize;
use std::path::{Path, PathBuf};
use tokio::{fs, io::AsyncWriteExt};
use warp::{self, http::StatusCode, multipart::FormData, Rejection, Reply};

// Find the downloads directory path for each OS and save files in the downloads directory
pub async fn save_file(mut form: FormData) -> Result<impl Reply, Rejection> {
    let upload_dir: PathBuf = download_dir().expect("Could not locate the Downloads directory");

    fs::create_dir_all(&upload_dir).await.map_err(|e| {
        debug_log!("Failed to create directory: {}", e);
        warp::reject::custom(super::InternalServerError)
    })?;

    while let Ok(Some(part)) = form.try_next().await {
        if let Some(filename) = part.filename() {
            let safe_filename = sanitize(filename);
            let mut unique_filepath = upload_dir.join(&safe_filename);
            let mut counter = 1;

            while fs::metadata(&unique_filepath).await.is_ok() {
                let stem = Path::new(&safe_filename)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("file");
                let ext = Path::new(&safe_filename)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");

                let new_filename = if ext.is_empty() {
                    format!("{} ({})", stem, counter)
                } else {
                    format!("{} ({}).{}", stem, counter, ext)
                };

                unique_filepath = upload_dir.join(new_filename);
                counter += 1;
            }

            debug_log!("Saving uploaded file to: {}", unique_filepath.display());

            let mut file = fs::File::create(&unique_filepath).await.map_err(|e| {
                debug_log!(
                    "Failed to create file '{}': {}",
                    unique_filepath.display(),
                    e
                );
                warp::reject::custom(super::InternalServerError)
            })?;

            let mut part_stream = part.stream();
            while let Some(chunk_result) = part_stream.next().await {
                let mut chunk = chunk_result.map_err(|e| {
                    debug_log!("Failed to read chunk from stream: {}", e);
                    warp::reject::custom(super::InternalServerError)
                })?;

                while chunk.has_remaining() {
                    let bytes = chunk.chunk();
                    file.write_all(bytes).await.map_err(|e| {
                        debug_log!("Failed to write chunk to file: {}", e);
                        warp::reject::custom(super::InternalServerError)
                    })?;
                    let len = bytes.len();
                    chunk.advance(len);
                }
            }
        }
    }

    Ok(warp::reply::with_status(
        "File(s) uploaded successfully.",
        StatusCode::CREATED,
    ))
}

// Generate TLS certificates required for warp https server and save them to disk
pub fn generate_tls_certs_to_disk() -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    // Generate a self-signed certificates
    let subject_alt_names = vec!["localhost".to_string()];
    let cert_key = generate_simple_self_signed(subject_alt_names)?;

    // Export to PEM format
    let cert_pem = cert_key.cert.pem();
    let key_pem = cert_key.signing_key.serialize_pem(); // Correct method

    // Write to ./certs next to executable
    let exe_dir = std::env::temp_dir();
    let certs_dir = exe_dir.join("certs");
    std::fs::create_dir_all(&certs_dir)?;

    let cert_path = certs_dir.join("cert.pem");
    let key_path = certs_dir.join("key.pem");

    std::fs::write(&cert_path, cert_pem)?;
    std::fs::write(&key_path, key_pem)?;

    debug_log!("🔐 TLS certificate written to: {}", cert_path.display());
    debug_log!("🔑 TLS private key written to: {}", key_path.display());

    Ok((cert_path, key_path))
}
