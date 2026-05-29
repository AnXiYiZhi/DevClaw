use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use p256::pkcs8::DecodePublicKey;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

const PUBLIC_KEY_PEM: &str = "-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEt3WS/ucAL3rtLLqTME6Tt5oWyeZ4\nUus3R2yYzA1nL1+Gc4Z1q+U8EuDH+tUfUerhzgume3jcK1EBvCRXTNCT8A==\n-----END PUBLIC KEY-----";

fn get_license_path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    dir.join("license.dat")
}

fn get_master_path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    dir.join("license.master")
}

#[derive(Serialize, Deserialize)]
struct LicensePayload {
    deviceId: String,
    expireTs: u64,
    v: u32,
}

#[derive(Serialize, Deserialize)]
struct LicenseData {
    payload: String,
    sig: String,
}

fn derive_key(device_id: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"devclaw-license-aes-key");
    hasher.update(device_id.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

fn decrypt_license(encrypted: &[u8], device_id: &str) -> Option<String> {
    if encrypted.len() < 12 {
        return None;
    }
    let (nonce_bytes, ciphertext) = encrypted.split_at(12);
    let key = derive_key(device_id);
    let cipher = Aes256Gcm::new_from_slice(&key).ok()?;
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext).ok()
        .and_then(|b| String::from_utf8(b).ok())
}

fn read_and_verify_stored_license(app: &tauri::AppHandle) -> bool {
    // 检查最高密钥
    if get_master_path(&app).exists() {
        return true;
    }

    let license_path = get_license_path(&app);
    let encrypted = match fs::read(&license_path) {
        Ok(data) => data,
        Err(_) => return false,
    };

    let device_id = get_device_id_internal();
    let license_json = match decrypt_license(&encrypted, &device_id) {
        Some(json) => json,
        None => return false,
    };

    let data: LicenseData = match serde_json::from_str(&license_json) {
        Ok(d) => d,
        Err(_) => return false,
    };

    verify_signature(&data.payload, &data.sig).is_ok()
}

fn get_device_id_internal() -> String {
    use sysinfo::{DiskKind, Disks, System};

    let mut sys = System::new();
    sys.refresh_cpu();
    sys.refresh_memory();

    let mut components = Vec::new();

    // CPU 品牌
    if let Some(cpu) = sys.cpus().first() {
        components.push(cpu.brand().to_string());
    }

    // 物理磁盘信息（类型+名称+总量）
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

fn verify_signature(payload: &str, sig_b64: &str) -> Result<(), String> {
    let verifying_key = VerifyingKey::from_public_key_pem(PUBLIC_KEY_PEM)
        .map_err(|e| format!("公钥解析失败: {e}"))?;

    let sig_bytes = URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| "签名 base64 解码失败".to_string())?;

    let signature = Signature::from_slice(&sig_bytes)
        .map_err(|e| format!("签名格式错误: {e}"))?;

    verifying_key
        .verify(payload.as_bytes(), &signature)
        .map_err(|_| "签名验证失败".to_string())
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
pub fn verify_license(app: tauri::AppHandle, license: String) -> VerifyResult {
    // 0. 最高密钥直接通过
    if license.trim() == "123456" {
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

    // 1. Base64 解码
    let license_json = match URL_SAFE_NO_PAD.decode(license.trim()) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => {
                return VerifyResult {
                    ok: false,
                    error: Some(format!("激活码格式错误: {e}")),
                };
            }
        },
        Err(e) => {
            return VerifyResult {
                ok: false,
                error: Some(format!("激活码 base64 解码失败: {e}")),
            };
        }
    };

    // 2. 解析 JSON
    let data: LicenseData = match serde_json::from_str(&license_json) {
        Ok(d) => d,
        Err(e) => {
            return VerifyResult {
                ok: false,
                error: Some(format!("激活码 JSON 解析失败: {e}")),
            };
        }
    };

    // 3. 验证签名
    if let Err(e) = verify_signature(&data.payload, &data.sig) {
        return VerifyResult {
            ok: false,
            error: Some(format!("签名验证失败: {e}")),
        };
    }

    // 4. 解析 payload
    let payload: LicensePayload = match serde_json::from_str(&data.payload) {
        Ok(p) => p,
        Err(e) => {
            return VerifyResult {
                ok: false,
                error: Some(format!("激活码 payload 解析失败: {e}")),
            };
        }
    };

    // 5. 验证设备 ID
    let device_id = get_device_id_internal();
    if payload.deviceId != device_id {
        return VerifyResult {
            ok: false,
            error: Some("此激活码与当前设备不匹配".to_string()),
        };
    }

    // 6. 验证过期时间
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if payload.expireTs < now {
        return VerifyResult {
            ok: false,
            error: Some("此激活码已过期".to_string()),
        };
    }

    // 7. 加密存储
    let license_path = get_license_path(&app);
    if let Some(parent) = license_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    // 检查是否与已存储的相同
    let device_id = get_device_id_internal();
    if let Ok(existing) = fs::read(&license_path) {
        if let Some(existing_json) = decrypt_license(&existing, &device_id) {
            if existing_json == license_json {
                log::info!("✓ License stored (same as existing)");
                return VerifyResult {
                    ok: true,
                    error: None,
                };
            }
        }
    }

    let key = derive_key(&device_id);
    let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    match cipher.encrypt(nonce, license_json.as_bytes()) {
        Ok(ciphertext) => {
            let mut output = Vec::with_capacity(12 + ciphertext.len());
            output.extend_from_slice(&nonce_bytes);
            output.extend_from_slice(&ciphertext);
            if let Err(e) = fs::write(&license_path, output) {
                log::error!("写入 license 文件失败: {e}");
            } else {
                log::info!("✓ License stored at {:?}", license_path);
            }
        }
        Err(e) => {
            log::error!("加密 license 失败: {e}");
        }
    }

    VerifyResult {
        ok: true,
        error: None,
    }
}
