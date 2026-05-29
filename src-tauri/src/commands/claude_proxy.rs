#![allow(non_snake_case)]

use serde_json::Value;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

fn claude_settings_path() -> std::path::PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    home.join(".claude").join("settings.json")
}

fn read_claude_settings() -> Result<Value, String> {
    let path = claude_settings_path();
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取配置失败: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("解析配置失败: {e}"))
}

fn write_claude_settings(settings: &Value) -> Result<(), String> {
    let path = claude_settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    let content = serde_json::to_string_pretty(settings).map_err(|e| format!("序列化失败: {e}"))?;
    std::fs::write(&path, content).map_err(|e| format!("写入配置失败: {e}"))
}

#[derive(serde::Serialize)]
pub struct ClaudeProxyInfo {
    pub http_port: String,
    pub socks5_port: String,
    pub env_vars: Value,
}

fn extract_port(url: &str, scheme: &str) -> String {
    let remainder = url.trim().trim_start_matches(scheme).trim_start_matches("://");
    let remainder = remainder.trim_end_matches('/');
    remainder
        .splitn(2, ':')
        .nth(1)
        .unwrap_or("")
        .to_string()
}

#[tauri::command]
pub async fn get_claude_proxy() -> Result<Option<ClaudeProxyInfo>, String> {
    let settings = read_claude_settings()?;
    let env = match settings.get("env") {
        Some(e) if e.is_object() => e,
        _ => return Ok(None),
    };

    let has_proxy = env.get("HTTP_PROXY").or_else(|| env.get("HTTPS_PROXY")).or_else(|| env.get("ALL_PROXY"));
    match has_proxy {
        Some(v) if v.as_str().map(|s| !s.is_empty()).unwrap_or(false) => {}
        _ => return Ok(None),
    }

    let http_port = env
        .get("HTTP_PROXY")
        .or_else(|| env.get("HTTPS_PROXY"))
        .and_then(|v| v.as_str())
        .map(|s| extract_port(s, "http"))
        .unwrap_or_default();

    let socks5_port = env
        .get("ALL_PROXY")
        .and_then(|v| v.as_str())
        .filter(|s| s.starts_with("socks5"))
        .map(|s| extract_port(s, "socks5"))
        .unwrap_or_default();

    Ok(Some(ClaudeProxyInfo {
        http_port,
        socks5_port,
        env_vars: env.clone(),
    }))
}

#[tauri::command]
pub async fn set_claude_proxy(
    http_port: String,
    socks5_port: String,
) -> Result<(), String> {
    let mut settings = read_claude_settings()?;
    if !settings.is_object() {
        settings = serde_json::json!({});
    }

    let env = settings
        .as_object_mut()
        .unwrap()
        .entry("env")
        .or_insert_with(|| serde_json::json!({}));

    if let Some(env_obj) = env.as_object_mut() {
        if !http_port.is_empty() {
            let url = format!("http://127.0.0.1:{}", http_port);
            env_obj.insert("HTTP_PROXY".to_string(), Value::String(url.clone()));
            env_obj.insert("HTTPS_PROXY".to_string(), Value::String(url));
        }
        if !socks5_port.is_empty() {
            let url = format!("socks5://127.0.0.1:{}", socks5_port);
            env_obj.insert("ALL_PROXY".to_string(), Value::String(url));
        }
    }

    write_claude_settings(&settings)
}

#[tauri::command]
pub async fn clear_claude_proxy() -> Result<(), String> {
    let mut settings = read_claude_settings()?;
    if let Some(env) = settings.get_mut("env").and_then(|v| v.as_object_mut()) {
        env.remove("HTTP_PROXY");
        env.remove("HTTPS_PROXY");
        env.remove("ALL_PROXY");
        if env.is_empty() {
            settings.as_object_mut().unwrap().remove("env");
        }
    }
    write_claude_settings(&settings)
}

#[tauri::command]
pub async fn test_proxy(proxy_type: String, host: String, port: String) -> Result<String, String> {
    let scheme = match proxy_type.as_str() {
        "socks5" => "socks5",
        "https" => "https",
        _ => "http",
    };
    let proxy_url = format!("{}://{}:{}", scheme, host, port);

    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| format!("代理配置无效: {e}"))?)
        .timeout(std::time::Duration::from_secs(15))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {e}"))?;

    let resp = client
        .get("https://www.google.com")
        .send()
        .await
        .map_err(|e| format!("连接失败: {e}"))?;

    let status = resp.status();
    if status.is_success() || status.is_redirection() {
        Ok("代理连接正常".to_string())
    } else {
        Err(format!("代理响应异常: HTTP {}", status.as_u16()))
    }
}

#[tauri::command]
pub async fn get_current_ip(proxy_type: Option<String>, host: Option<String>, port: Option<String>) -> Result<String, String> {
    let client_builder = reqwest::Client::builder().timeout(std::time::Duration::from_secs(10));

    let client = if let (Some(pt), Some(h), Some(p)) = (proxy_type, host, port) {
        let scheme = match pt.as_str() {
            "socks5" => "socks5",
            "https" => "https",
            _ => "http",
        };
        let proxy_url = format!("{}://{}:{}", scheme, h, p);
        client_builder
            .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| format!("代理配置无效: {e}"))?)
            .build()
            .map_err(|e| format!("创建客户端失败: {e}"))?
    } else {
        client_builder.build().map_err(|e| format!("创建客户端失败: {e}"))?
    };

    let resp = client
        .get("https://ipinfo.io/json")
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;

    let body: Value = resp
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {e}"))?;

    let ip = body.get("ip").and_then(|v| v.as_str()).unwrap_or("未知");
    let city = body.get("city").and_then(|v| v.as_str()).unwrap_or("");
    let region = body.get("region").and_then(|v| v.as_str()).unwrap_or("");
    let country = body.get("country").and_then(|v| v.as_str()).unwrap_or("");
    let org = body.get("org").and_then(|v| v.as_str()).unwrap_or("");

    let location = [city, region, country]
        .iter()
        .filter(|s| !s.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");

    Ok(format!("{}\n{}\n{}", ip, location, org))
}

#[tauri::command]
pub async fn open_claude_dir(handle: AppHandle) -> Result<bool, String> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let claude_dir = home.join(".claude");

    if !claude_dir.exists() {
        std::fs::create_dir_all(&claude_dir).map_err(|e| format!("创建目录失败: {e}"))?;
    }

    handle
        .opener()
        .open_path(claude_dir.to_string_lossy().to_string(), None::<String>)
        .map_err(|e| format!("打开文件夹失败: {e}"))?;

    Ok(true)
}
