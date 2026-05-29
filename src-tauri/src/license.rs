use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

const API_BASE: &str = "https://devclaw.ccwu.cc";
const MASTER_PASSWORD: &str = "123456";

fn get_license_path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    dir.join("license.json")
}

fn get_master_path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    dir.join("license.master")
}

#[derive(Serialize, Deserialize, Clone)]
struct StoredLicense {
    key: String,
    #[serde(default)]
    expire_ts: Option<u64>,
}

#[derive(Serialize)]
struct VerifyKeyRequest {
    #[serde(rename = "licenseKey")]
    license_key: String,
    #[serde(rename = "deviceId")]
    device_id: String,
}

#[derive(Deserialize)]
struct VerifyKeyResponse {
    success: bool,
    #[serde(alias = "message")]
    error: Option<String>,
}

fn get_device_id_internal() -> String {
    use sysinfo::{DiskKind, Disks, System};

    let mut sys = System::new();
    sys.refresh_cpu();
    sys.refresh_memory();

    let mut components = Vec::new();

    if let Some(cpu) = sys.cpus().first() {
        components.push(cpu.brand().to_string());
    }

    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        if matches!(disk.kind(), DiskKind::HDD | DiskKind::SSD) {
            let name = disk.name().to_string_lossy().to_string();
            let total = disk.total_space();
            components.push(format!("{name}:{total}"));
        }
    }

    let combined = components.join("|");
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    hex::encode(hasher.finalize())
}

fn is_valid_key_format(key: &str) -> bool {
    let re = Regex::new(r"^CCS-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}$").unwrap();
    re.is_match(key)
}

fn save_license(app: &tauri::AppHandle, stored: &StoredLicense) {
    let path = get_license_path(app);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match serde_json::to_string(stored) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                log::error!("写入 license 文件失败: {e}");
            } else {
                log::info!("✓ License stored at {:?}", path);
            }
        }
        Err(e) => log::error!("序列化 license 失败: {e}"),
    }
}

fn read_and_verify_stored_license(app: &tauri::AppHandle) -> bool {
    // 本地最高密码
    if get_master_path(app).exists() {
        return true;
    }

    let path = get_license_path(app);
    let data = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let stored: StoredLicense = match serde_json::from_str(&data) {
        Ok(d) => d,
        Err(_) => return false,
    };

    // 格式校验
    if !is_valid_key_format(&stored.key) {
        return false;
    }

    // 过期检查（如果有过期时间）
    if let Some(expire_ts) = stored.expire_ts {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if now > expire_ts {
            return false;
        }
    }

    true
}

#[tauri::command]
pub fn get_device_id() -> String {
    get_device_id_internal()
}

#[tauri::command]
pub fn check_license(app: tauri::AppHandle) -> bool {
    read_and_verify_stored_license(&app)
}

#[derive(Serialize)]
pub struct VerifyResult {
    ok: bool,
    error: Option<String>,
}

#[tauri::command]
pub async fn verify_license(app: tauri::AppHandle, license: String) -> VerifyResult {
    let key = license.trim().to_string();

    // 1. 本地最高密码直接通过
    if key == MASTER_PASSWORD {
        let master_path = get_master_path(&app);
        if let Some(parent) = master_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = fs::write(&master_path, "master") {
            log::error!("写入 master key 文件失败: {e}");
        } else {
            log::info!("✓ Master key accepted, stored at {:?}", master_path);
        }
        return VerifyResult {
            ok: true,
            error: None,
        };
    }

    // 2. 格式校验: CCS-XXXX-XXXX-XXXX
    if !is_valid_key_format(&key) {
        return VerifyResult {
            ok: false,
            error: Some("密钥格式错误，应为 CCS-XXXX-XXXX-XXXX".to_string()),
        };
    }

    // 3. 请求服务器验证
    let device_id = get_device_id_internal();
    let client = reqwest::Client::new();
    let resp = match client
        .post(format!("{}/api/verify-key", API_BASE))
        .json(&VerifyKeyRequest {
            license_key: key.clone(),
            device_id,
        })
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return VerifyResult {
                ok: false,
                error: Some(format!("网络请求失败: {e}")),
            };
        }
    };

    let result: VerifyKeyResponse = match resp.json().await {
        Ok(r) => r,
        Err(e) => {
            return VerifyResult {
                ok: false,
                error: Some(format!("服务器响应解析失败: {e}")),
            };
        }
    };

    if result.success {
        save_license(&app, &StoredLicense {
            key,
            expire_ts: None,
        });
        VerifyResult {
            ok: true,
            error: None,
        }
    } else {
        VerifyResult {
            ok: false,
            error: Some(result.error.unwrap_or_else(|| "验证失败".to_string())),
        }
    }
}
