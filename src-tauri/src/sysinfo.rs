use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Serialize)]
pub struct SysInfo {
    pub os_type: Option<String>,
    pub hostname: Option<String>,
}

#[tauri::command]
pub fn get_sys_info() -> SysInfo {
    SysInfo {
        os_type: System::long_os_version(),
        hostname: System::host_name(),
    }
}
