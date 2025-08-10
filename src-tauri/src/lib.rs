mod http_requests;
mod http_server;
mod logger;
mod mdns;
mod sysinfo;
mod tcp_heartbeat;
mod util;
mod util_android;

use once_cell::sync::OnceCell;
use tauri::AppHandle;

#[cfg(target_os = "android")]
use crate::util_android::get_file_infos_with_previews;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use crate::util::get_file_infos_with_previews;

static APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_android_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(util::prevent_default())
        .invoke_handler(tauri::generate_handler![
            sysinfo::get_sys_info,
            mdns::start_mdns_responder,
            mdns::stop_mdns_responder,
            mdns::restart_mdns_responder,
            mdns::discover_mdns_services,
            http_server::start_http_server,
            http_server::respond_to_request,
            util::get_primary_ipv4,
            get_file_infos_with_previews,
            tcp_heartbeat::add_device,
            tcp_heartbeat::remove_device,
            http_requests::assisted_discovery,
            http_requests::file_transfer_request,
            http_requests::upload_files
        ])
        .setup(|app| {
            // Store the actual app handle in your global
            APP_HANDLE.set(app.handle().clone()).unwrap();

            tcp_heartbeat::start_heartbeat_responder();

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tcp_heartbeat::start_heartbeat(app_handle).await;
            });

            mdns::listen_for_mdns_services(app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
