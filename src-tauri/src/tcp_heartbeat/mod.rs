use crate::debug_log;
use once_cell::sync::Lazy;
use std::{collections::HashMap, net::SocketAddr, time::Duration};
use tauri::async_runtime::spawn;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{lookup_host, TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::timeout;

/// Represents a device tracked by heartbeat server.
#[derive(Debug)]
struct TrackedDevice {
    ip: String,
    is_online: bool,
    stream: Option<TcpStream>, // Async TcpStream wrapped in Option
}

// Shared device map protected by async Mutex.
type DiscoveredDevices = Lazy<Mutex<HashMap<String, TrackedDevice>>>;

// Shared global device list.
static DEVICES: DiscoveredDevices = Lazy::new(|| Mutex::new(HashMap::new()));

// TCP heartbeat server port.
const PORT: u16 = 21112;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(10);

// add a device to global device list
// front-end will call add device when it discovers any device
#[tauri::command]
pub async fn add_device(ip: String) {
    let mut devices = DEVICES.lock().await;
    devices.insert(
        ip.clone(),
        TrackedDevice {
            ip,
            is_online: false,
            stream: None,
        },
    );
}

// remove a device to global device list
#[tauri::command]
pub async fn remove_device(ip: String) {
    let mut devices = DEVICES.lock().await;
    devices.remove(&ip);
}

#[tauri::command]
pub async fn start_heartbeat(app: AppHandle) {
    debug_log!("💓 Started sending heartbeats to discovered devices.");

    let app_for_task = app.clone();

    spawn(async move {
        loop {
            let ips: Vec<String> = {
                let devices = DEVICES.lock().await;
                devices.keys().cloned().collect()
            };

            for ip in ips {
                let stream = {
                    let mut devices = DEVICES.lock().await;
                    match devices.get_mut(&ip) {
                        Some(dev) => dev.stream.take(),
                        None => continue,
                    }
                };

                let mut stream = match stream {
                    Some(s) => s,
                    None => {
                        let addr_str = format!("{}:{}", ip, PORT);
                        let addrs: Vec<SocketAddr> = match lookup_host(addr_str.as_str()).await {
                            Ok(iter) => iter.collect(),
                            Err(e) => {
                                debug_log!("Could not resolve {}: {}", ip, e);
                                mark_device_offline(&app_for_task, &ip).await;
                                continue;
                            }
                        };

                        let socket_addr = match addrs.first() {
                            Some(a) => *a,
                            None => {
                                debug_log!("No valid socket address for {}", ip);
                                mark_device_offline(&app_for_task, &ip).await;
                                continue;
                            }
                        };

                        match timeout(HEARTBEAT_TIMEOUT, TcpStream::connect(socket_addr)).await {
                            Ok(Ok(s)) => s,
                            Ok(Err(e)) => {
                                debug_log!("Connection error to {}: {}", ip, e);
                                mark_device_offline(&app_for_task, &ip).await;
                                continue;
                            }
                            Err(_) => {
                                debug_log!("Connection to {} timed out", ip);
                                mark_device_offline(&app_for_task, &ip).await;
                                continue;
                            }
                        }
                    }
                };

                let mut stream_ok = true;
                let mut saw_pong = false;

                if let Err(e) = stream.write_all(b"ping\n").await {
                    debug_log!("Write failed to {}: {}", ip, e);
                    stream_ok = false;
                }

                if stream_ok {
                    let mut reader = BufReader::new(&mut stream);
                    let mut response = String::new();

                    match timeout(HEARTBEAT_TIMEOUT, reader.read_line(&mut response)).await {
                        Ok(Ok(0)) => {
                            debug_log!("Connection closed by {}", ip);
                            stream_ok = false;
                        }
                        Ok(Ok(_)) => {
                            if response.trim().starts_with("pong") {
                                saw_pong = true;
                            } else {
                                debug_log!("Unexpected response from {}: {}", ip, response.trim());
                                stream_ok = false;
                            }
                        }
                        Ok(Err(e)) => {
                            debug_log!("Read error from {}: {}", ip, e);
                            stream_ok = false;
                        }
                        Err(_) => {
                            debug_log!("Read timed out from {}", ip);
                            stream_ok = false;
                        }
                    }
                }

                {
                    let mut devices = DEVICES.lock().await;
                    if let Some(dev) = devices.get_mut(&ip) {
                        if stream_ok {
                            dev.stream = Some(stream);
                            if !dev.is_online && saw_pong {
                                dev.is_online = true;
                                let _ = app_for_task.emit("device-online", dev.ip.clone());
                            }
                        } else {
                            dev.stream = None;
                            if dev.is_online {
                                dev.is_online = false;
                                let _ = app_for_task.emit("device-offline", dev.ip.clone());
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(HEARTBEAT_INTERVAL).await;
        }
    });
}

// mark the device as offline in the global list of devices and emit event to front-end so it can
// remove it from listed devices
async fn mark_device_offline(app: &AppHandle, ip: &str) {
    let mut devices = DEVICES.lock().await;
    if let Some(dev) = devices.get_mut(ip) {
        dev.stream = None;
        if dev.is_online {
            dev.is_online = false;
            let _ = app.emit("device-offline", dev.ip.clone());
        }
    }
}

// start the tcp heartbeat responder, it will reply with "pong" for every "ping" it receives
// ping/pong messages will be terminated by \n
pub fn start_heartbeat_responder() {
    spawn(async move {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT))
            .await
            .expect("Failed to bind TCP listener");

        debug_log!("💓 Heartbeat responder listening on port {}", PORT);

        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    debug_log!("Incoming connection from {}", peer);
                    spawn(handle_connection(stream));
                }
                Err(e) => debug_log!("Error accepting connection: {}", e),
            }
        }
    });
}

async fn handle_connection(mut stream: TcpStream) {
    let peer = stream
        .peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| "[unknown]".into());

    let (reader, mut writer) = stream.split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        match buf_reader.read_line(&mut line).await {
            Ok(0) => {
                debug_log!("Connection closed by {}", peer);
                break;
            }
            Ok(_) => {
                debug_log!("Received {} from {}", line.trim(), peer);
                if line.trim().starts_with("ping") {
                    if let Err(e) = writer.write_all(b"pong\n").await {
                        debug_log!("Failed to send pong to {}: {}", peer, e);
                        break;
                    }
                    debug_log!("Sent pong to {}", peer);
                }
            }
            Err(e) => {
                debug_log!("Error reading from {}: {}", peer, e);
                break;
            }
        }
    }
}
