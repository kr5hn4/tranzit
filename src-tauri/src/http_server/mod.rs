mod utils;
mod utils_android;

use crate::debug_log;
use crate::mdns::Peer;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;
use uuid::Uuid;
use warp::{self, http::Method, http::StatusCode, reject::Reject, Filter, Rejection, Reply};

type ResponseSender = oneshot::Sender<String>;

static PENDING_REQUESTS: Lazy<Arc<Mutex<HashMap<String, ResponseSender>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

// Custom internal error used for returning 5xx
#[derive(Debug)]
pub struct InternalServerError;

impl Reject for InternalServerError {}

// Convert custom rejections into proper HTTP responses
async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if err.find::<InternalServerError>().is_some() {
        Ok(warp::reply::with_status(
            "Internal Server Error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if err.is_not_found() {
        Ok(warp::reply::with_status("Not Found", StatusCode::NOT_FOUND))
    } else if let Some(_) = err.find::<warp::reject::PayloadTooLarge>() {
        Ok(warp::reply::with_status(
            "Payload too large",
            StatusCode::PAYLOAD_TOO_LARGE,
        ))
    } else {
        debug_log!("Unhandled rejection: {:?}", err);
        Ok(warp::reply::with_status(
            "Internal Server Error",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FilesInfo {
    name: String,
    size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DeviceInfo {
    hostname: String,
    os_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FileTransferRequest {
    files_info: Vec<FilesInfo>,
    device_info: DeviceInfo,
    receiver_info: String,
}

#[tauri::command]
pub fn start_http_server(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Generate TLS certs and write them to disk (cert.pem & key.pem)
        #[cfg(target_os = "android")]
        let (cert_path, key_path) =
            utils_android::generate_tls_certs_to_disk(&app).unwrap_or_else(|e| {
                debug_log!("Failed to generate TLS certs on Android: {}", e);
                std::process::exit(1);
            });

        // Generate TLS certs and write them to disk (cert.pem & key.pem)
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let (cert_path, key_path) = utils::generate_tls_certs_to_disk().unwrap_or_else(|e| {
            debug_log!("Failed to generate TLS certs: {}", e);
            std::process::exit(1);
        });

        let cert_bytes = std::fs::read(&cert_path).expect("Failed to read TLS cert");
        let key_bytes = std::fs::read(&key_path).expect("Failed to read TLS key");

        // Configure CORS
        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(&[Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(vec!["Content-Type"]);

        // assisted-discovery route
        let app_ = app.clone();
        let post_info = warp::path("assisted-discovery")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |device_info: Peer| {
                debug_log!("received assisted discovery request");
                match app_.emit("assisted-discovery", &device_info) {
                    Ok(_) => debug_log!("Assisted discovery Event emitted successfully!"),
                    Err(e) => debug_log!("Failed to emit assisted discovery event: {}", e),
                }
                warp::reply::json(&"Device info received")
            });

        // file-transfer-request route
        let my_endpoint_route = warp::path!("file-transfer-request")
            .and(warp::post())
            .and(warp::body::json())
            .and_then({
                let _app = app.clone();
                move |req_body: FileTransferRequest| {
                    let app = _app.clone();
                    async move {
                        let (tx, rx) = oneshot::channel::<String>();
                        let req_id = Uuid::new_v4().to_string();

                        PENDING_REQUESTS.lock().unwrap().insert(req_id.clone(), tx);

                        app.emit(
                            "file-transfer-request",
                            serde_json::json!({
                                "id": req_id.clone(),
                                "data": req_body
                            }),
                        )
                        .unwrap();

                        let response = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
                            .await
                            .map_err(|_| warp::reject::custom(InternalServerError))?
                            .map_err(|_| warp::reject::custom(InternalServerError))?;

                        Ok::<_, Rejection>(warp::reply::json(&response))
                    }
                }
            });

        #[cfg(target_os = "android")]
        let save_file_fn = utils_android::save_file;

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let save_file_fn = utils::save_file;

        // upload route
        let upload_route = warp::path!("upload")
            .and(warp::post())
            // accept multipart form data and set max uploadable file size to 5GB
            .and(warp::multipart::form().max_length(5000 * 1024 * 1024))
            .and_then(save_file_fn);

        // combine all routes
        let routes = my_endpoint_route
            .or(upload_route)
            .or(post_info)
            .with(cors)
            .recover(handle_rejection);

        let port: u16 = match env::var("HTTPS_PORT") {
            Ok(val) => match val.parse() {
                Ok(num) => num,
                Err(_) => 21212,
            },
            Err(_) => {
                debug_log!("HTTPS_PORT environment variable not set, falling back to 21212");
                21212
            }
        };
        debug_log!("🔒🖥️ Starting HTTPS server on https://0.0.0.0:21212");
        // start the https server
        warp::serve(routes)
            .tls()
            .cert(cert_bytes)
            .key(key_bytes)
            .run(([0, 0, 0, 0], port))
            .await;
    });
}

#[tauri::command]
pub async fn respond_to_request(id: String, data: String) {
    if let Some(sender) = PENDING_REQUESTS.lock().unwrap().remove(&id) {
        let _ = sender.send(data);
    }
}
