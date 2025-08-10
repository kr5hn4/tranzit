use crate::debug_log;
use crate::sysinfo;
use libmdns::Responder;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    net::IpAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    thread::JoinHandle,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

pub static APP_UUID: Lazy<String> = Lazy::new(|| Uuid::new_v4().to_string());

// Globals to manage mDNS responder thread and running state
static MDNS_THREAD: Lazy<Mutex<Option<JoinHandle<()>>>> = Lazy::new(|| Mutex::new(None));
static SHOULD_RUN: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));

// Starts the mDNS responder in a background thread
#[tauri::command]
pub fn start_mdns_responder() {
    let mut thread_handle = match MDNS_THREAD.lock() {
        Ok(guard) => guard,
        Err(e) => {
            debug_log!("Failed to lock MDNS_THREAD mutex: {}", e);
            return;
        }
    };

    if SHOULD_RUN.load(Ordering::SeqCst) {
        debug_log!("mDNS responder already running!");
        return;
    }

    SHOULD_RUN.store(true, Ordering::SeqCst);
    let should_run = Arc::clone(&SHOULD_RUN);

    *thread_handle = Some(thread::spawn(move || {
        let responder = match Responder::new() {
            Ok(r) => r,
            Err(e) => {
                debug_log!("Failed to start responder: {}", e);
                SHOULD_RUN.store(false, Ordering::SeqCst);
                return;
            }
        };

        let sys_info = sysinfo::get_sys_info();

        let os_type = sys_info.os_type.unwrap_or_else(|| "unknown".into());
        let hostname = sys_info.hostname.unwrap_or_else(|| "unknown".into());

        let txt_records = vec![
            "version=1.0".to_string(),
            format!("os={}", os_type),
            format!("hostname={}", hostname),
            format!("arch={}", std::env::consts::ARCH),
            format!("id={}", APP_UUID.to_string()),
        ];

        let _svc = responder.register(
            "_localdrop._tcp".to_string(),
            "LocalDrop Peer".to_string(),
            21212,
            &txt_records.iter().map(|s| &**s).collect::<Vec<&str>>(),
        );

        debug_log!("mDNS service registered");

        while should_run.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(2));
        }

        debug_log!("mDNS responder thread exiting");
    }));
}

/// Stops the mDNS responder thread
#[tauri::command]
pub fn stop_mdns_responder() {
    SHOULD_RUN.store(false, Ordering::SeqCst);

    match MDNS_THREAD.lock() {
        Ok(mut guard) => {
            if let Some(handle) = guard.take() {
                if handle.join().is_ok() {
                    debug_log!("mDNS responder stopped");
                } else {
                    debug_log!("Failed to join mDNS responder thread");
                }
            } else {
                debug_log!("mDNS responder was not running");
            }
        }
        Err(e) => {
            debug_log!("Failed to lock MDNS_THREAD mutex: {}", e);
        }
    }
}

/// Restart mDNS responder (stop then start)
#[tauri::command]
pub fn restart_mdns_responder() {
    stop_mdns_responder();
    start_mdns_responder();
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Peer {
    name: String,
    ip: IpAddr,
    port: u16,
    hostname: String,
    service_type: String,
    os: String,
    id: String,
}

// Implement hash + equality based on IP + port to avoid duplicates
impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip && self.port == other.port
    }
}
impl Eq for Peer {}
impl Hash for Peer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ip.hash(state);
        self.port.hash(state);
    }
}

/// Asynchronous mDNS discovery that runs blocking code on Tauri's async runtime
#[tauri::command]
pub async fn discover_mdns_services() -> Result<Vec<Peer>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        debug_log!("🔍 Starting mDNS discovery for _localdrop._tcp.local.");

        let mdns = ServiceDaemon::new().map_err(|e| format!("Failed to create daemon: {}", e))?;
        let receiver = mdns
            .browse("_localdrop._tcp.local.")
            .map_err(|e| format!("Failed to browse: {}", e))?;

        let timeout = Duration::from_secs(2);
        let start = Instant::now();
        let mut discovered_peers = HashSet::new();

        while start.elapsed() < timeout {
            match receiver.recv_timeout(Duration::from_millis(500)) {
                Ok(ServiceEvent::ServiceResolved(info)) => {
                    let props = info.get_properties();
                    let os = props
                        .get("os")
                        .map(|p| p.val_str().to_string())
                        .unwrap_or_else(|| "unknown".into());
                    let hostname = props
                        .get("hostname")
                        .map(|p| p.val_str().to_string())
                        .unwrap_or_else(|| "unknown".into());
                    let id = props
                        .get("id")
                        .map(|p| p.val_str().to_string())
                        .unwrap_or_else(|| "unknown".into());

                    for ip in info.get_addresses() {
                        if ip.is_ipv4() {
                            let peer = Peer {
                                name: info.get_fullname().to_string(),
                                ip: *ip,
                                port: info.get_port(),
                                hostname: hostname.clone(),
                                service_type: info.get_type().to_string(),
                                os: os.clone(),
                                id: id.clone(),
                            };
                            debug_log!("🗺️ Discovered peer: {:?}", peer);
                            discovered_peers.insert(peer);
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    debug_log!("Error receiving mDNS event: {}", e);
                }
            }
        }

        let discovered_vec: Vec<Peer> = discovered_peers.iter().cloned().collect();

        debug_log!(
            "Discovery complete. Found {} unique peer(s).",
            discovered_vec.len()
        );

        Ok(discovered_vec)
    })
    .await
    .map_err(|e| format!("Thread panicked: {}", e))?
}

/// Passive mDNS listener that emits discovered peers as events to frontend
#[tauri::command]
pub fn listen_for_mdns_services(app_handle: AppHandle) {
    std::thread::spawn(move || {
        debug_log!("👂 Passive mDNS listener started.");

        let mdns = match ServiceDaemon::new() {
            Ok(daemon) => daemon,
            Err(e) => {
                debug_log!("Failed to create mDNS daemon: {}", e);
                return;
            }
        };

        let receiver = match mdns.browse("_localdrop._tcp.local.") {
            Ok(r) => r,
            Err(e) => {
                debug_log!("Failed to browse: {}", e);
                return;
            }
        };

        let mut discovered = HashSet::new();
        let mut consecutive_errors = 0;
        const MAX_ERRORS: usize = 5;

        loop {
            match receiver.recv() {
                Ok(ServiceEvent::ServiceResolved(info)) => {
                    consecutive_errors = 0;

                    let props = info.get_properties();
                    let os = props
                        .get("os")
                        .map(|p| p.val_str().to_string())
                        .unwrap_or_else(|| "unknown".into());
                    let hostname = props
                        .get("hostname")
                        .map(|p| p.val_str().to_string())
                        .unwrap_or_else(|| "unknown".into());

                    let id = props
                        .get("id")
                        .map(|p| p.val_str().to_string())
                        .unwrap_or_else(|| "unknown".into());

                    for ip in info.get_addresses() {
                        if ip.is_ipv4() {
                            let peer = Peer {
                                name: info.get_fullname().to_string(),
                                ip: *ip,
                                port: info.get_port(),
                                hostname: hostname.clone(),
                                service_type: info.get_type().to_string(),
                                os: os.clone(),
                                id: id.clone(),
                            };

                            if discovered.insert(peer.clone()) {
                                debug_log!("🗺️ Discovered peer: {:?}", peer);
                                if let Err(err) = app_handle.emit("mdns-peer-discovered", &peer) {
                                    debug_log!("Failed to emit peer: {}", err);
                                }
                            }
                        }
                    }
                }
                Ok(_) => {
                    consecutive_errors = 0;
                }
                Err(e) => {
                    consecutive_errors += 1;
                    debug_log!("Receiver error #{}: {}", consecutive_errors, e);

                    if consecutive_errors >= MAX_ERRORS {
                        debug_log!("Exceeded max consecutive errors, stopping listener.");
                        break;
                    }

                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }

        debug_log!("mDNS listener exited.");
    });
}
