use crate::debug_log;
use bytes::Bytes;
use futures_util::stream::Stream;
use futures_util::stream::TryStreamExt;
use futures_util::task::{Context, Poll};
use pin_project_lite::pin_project;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::{
    multipart::{Form, Part},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::pin::Pin;
use tauri::{AppHandle, Emitter};
use tauri_plugin_android_fs::{FileAccessMode, FileUri};
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio_util::io::ReaderStream;

// If a device can discover others via mDNS but isn’t discoverable itself,
// it can notify the devices it finds about its presence via assisted discovery
#[tauri::command]
pub async fn assisted_discovery(
    device_ip: String,
    service_type: String,
    hostname: String,
    os_type: String,
    port: u16,
    ipv4: String,
) -> Result<String, String> {
    let body = json!({
        "name": hostname,
        "ip": ipv4,
        "port": port,
        "hostname": hostname,
        "service_type": service_type,
        "os": os_type,
    });

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let client = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let url = format!("https://{}:{}/assisted-discovery", device_ip, port);

    match client.post(&url).headers(headers).json(&body).send().await {
        Ok(response) => match response.text().await {
            Ok(text) => Ok(text),
            Err(e) => Err(format!("Failed to read response text: {}", e)),
        },
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}

#[derive(Serialize, Deserialize)]
pub struct FileInfo {
    name: String,
    size: u64,
}
#[derive(Serialize, Deserialize)]
pub struct DeviceInfo {
    hostname: String,
    os_type: String,
}

// send a file transfer request with file names, size and device info of
// the device initiating the file transfer request
#[tauri::command]
pub async fn file_transfer_request(
    ip: String,
    port: u16,
    selected_files: Vec<FileInfo>,
    device_info: DeviceInfo,
) -> Result<serde_json::Value, String> {
    let url = format!("https://{}:{}/file-transfer-request", ip, port);
    debug_log!("Sending request to: {}", url);

    let body = json!({
        "files_info": selected_files,
        "device_info": device_info,
        "receiver_info": ip
    });

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
    let response = client
        .post(&url)
        .headers(headers)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let result_json = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(result_json)
}

#[derive(serde::Deserialize)]
pub struct FileUploadInfo {
    file_path: String,
    file_uuid: String,
    name: String,
}

// upload files once the file transfer request is accepted,
// also report back the progress to front-end while doing so
#[tauri::command]
pub async fn upload_files(
    files: Vec<FileUploadInfo>,
    ip: String,
    port: u16,
    app_handle: AppHandle,
) -> Result<(), String> {
    for file in files {
        let app_handle = app_handle.clone();
        let ip = ip.clone();

        tauri::async_runtime::spawn(async move {
            if let Err(err) = upload_file_with_progress(
                &file.file_path,
                &file.name,
                &file.file_uuid,
                &ip,
                port,
                &app_handle,
            )
            .await
            {
                debug_log!("Error uploading {}: {:?}", file.file_path, err);
            }
        });
    }

    Ok(())
}

pin_project! {
    pub struct ProgressStream<S> {
        #[pin]
        inner: S,
        uploaded: u64,
        total: u64,
        file_name: String,
        file_uuid: String,
        app_handle: tauri::AppHandle,
    }
}

impl<S> Stream for ProgressStream<S>
where
    S: Stream<Item = Result<Bytes, std::io::Error>>,
{
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        match this.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                *this.uploaded += chunk.len() as u64;
                let percent = ((*this.uploaded as f64 / *this.total as f64) * 100.0).round();

                let _ = this.app_handle.emit(
                    "upload-progress",
                    serde_json::json!({
                        "filename": *this.file_name,
                        "percent": percent,
                        "uuid": *this.file_uuid,
                    }),
                );

                Poll::Ready(Some(Ok(chunk)))
            }
            other => other,
        }
    }
}

#[tauri::command]
pub async fn upload_file_with_progress(
    file_path: &str,
    name: &str,
    file_uuid: &str,
    ip: &str,
    port: u16,
    app_handle: &tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    use tokio_stream::Stream;
    let file_name = String::from(name);
    let (stream, total_size, mime_type): (
        Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>,
        u64,
        String,
    ) = {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            let file = tokio::fs::File::open(file_path).await?;
            let metadata = file.metadata().await?;
            let total_size = metadata.len();

            let mime_type = infer::get_from_path(file_path)
                .ok()
                .flatten()
                .map(|kind| kind.mime_type().to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());

            let stream = FramedRead::new(file, BytesCodec::new())
                .map_ok(Bytes::from)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

            (Box::pin(stream), total_size, mime_type)
        }

        #[cfg(target_os = "android")]
        {
            let fs = app_handle.android_fs();
            let file_uri = FileUri {
                uri: file_path.to_string(),
                document_top_tree_uri: None,
            };

            let std_file = fs
                .open_file(&file_uri, FileAccessMode::Read)
                .map_err(|e| format!("Android file open failed: {e}"))?;

            let metadata = std_file
                .metadata()
                .map_err(|e| format!("Android file metadata error: {e}"))?;
            let total_size = metadata.len();

            let mime_type = fs
                .get_mime_type(&file_uri)
                .map_err(|e| format!("Android mime error: {e}"))?
                .unwrap_or_else(|| "application/octet-stream".into());

            // Just convert to async and stream
            let file = tokio::fs::File::from_std(std_file);

            let stream = ReaderStream::new(file)
                .map_ok(Bytes::from)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

            (Box::pin(stream), total_size, mime_type)
        }
    };

    let progress_stream = ProgressStream {
        inner: stream,
        uploaded: 0,
        total: total_size,
        file_name: file_name.clone(),
        file_uuid: file_uuid.to_string(),
        app_handle: app_handle.clone(),
    };

    let body = reqwest::Body::wrap_stream(progress_stream);

    let part = Part::stream_with_length(body, total_size)
        .file_name(file_name.clone())
        .mime_str(&mime_type)?;

    let form = Form::new().part("file", part);

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let url = format!("https://{}:{}/upload", ip, port);
    let response = client.post(&url).multipart(form).send().await?;

    debug_log!("Upload response for {}: {:?}", file_name, response.status());
    if response.status().is_success() {
        debug_log!("{} uploaded", file_name);
    } else {
        debug_log!("Failed to upload {}: {}", file_name, response.status());
    }

    Ok(())
}
