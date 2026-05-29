use std::collections::HashMap;
use std::process::Command;
use tauri::Emitter;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// 获取 shell 的完整 PATH
fn get_shell_path() -> String {
    let current = std::env::var("PATH").unwrap_or_default();

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("/bin/zsh")
            .args(["-l", "-c", "echo $PATH"])
            .output()
        {
            if output.status.success() {
                let shell_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !shell_path.is_empty() {
                    return format!("{}:{}", shell_path, current);
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("/bin/bash")
            .args(["-l", "-c", "echo $PATH"])
            .output()
        {
            if output.status.success() {
                let shell_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !shell_path.is_empty() {
                    return format!("{}:{}", shell_path, current);
                }
            }
        }
    }

    current
}

/// Windows: 通过 cmd /C 执行命令，正确处理 .cmd/.bat 文件
/// macOS/Linux: 直接执行
fn get_command_output(cmd: &str, args: &[&str]) -> Option<String> {
    let path = get_shell_path();
    log::debug!("[env_check] 执行: {} {:?}", cmd, args);

    #[cfg(target_os = "windows")]
    {
        // 拼接命令字符串: "cmd arg1 arg2"
        let full_cmd = std::iter::once(cmd)
            .chain(args.iter().copied())
            .collect::<Vec<_>>()
            .join(" ");
        let mut c = Command::new("cmd");
        c.creation_flags(0x08000000);
        let result = c
            .args(["/C", &full_cmd])
            .env("PATH", &path)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout)
                        .ok()
                        .map(|s| s.lines().next().unwrap_or("").trim().to_string())
                } else {
                    log::debug!("[env_check] {} stderr: {}", cmd, String::from_utf8_lossy(&o.stderr).trim());
                    None
                }
            })
            .filter(|s| !s.is_empty());
        log::debug!("[env_check] {} 结果: {:?}", cmd, result);
        result
    }

    #[cfg(not(target_os = "windows"))]
    {
        let result = Command::new(cmd)
            .args(args)
            .env("PATH", &path)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout)
                        .ok()
                        .map(|s| s.lines().next().unwrap_or("").trim().to_string())
                } else {
                    log::debug!("[env_check] {} stderr: {}", cmd, String::from_utf8_lossy(&o.stderr).trim());
                    None
                }
            })
            .filter(|s| !s.is_empty());
        log::debug!("[env_check] {} 结果: {:?}", cmd, result);
        result
    }
}

/// 通过完整路径执行命令（用于 Chrome 等不在 PATH 的工具）
fn get_command_output_at(exe_path: &str, args: &[&str]) -> Option<String> {
    let path = get_shell_path();
    log::debug!("[env_check] 执行(完整路径): {} {:?}", exe_path, args);
    let mut c = Command::new(exe_path);
    #[cfg(target_os = "windows")]
    c.creation_flags(0x08000000);
    let result = c
        .args(args)
        .env("PATH", &path)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.lines().next().unwrap_or("").trim().to_string())
            } else {
                None
            }
        })
        .filter(|s| !s.is_empty());
    log::debug!("[env_check] {} 结果: {:?}", exe_path, result);
    result
}

/// 通用工具检测（node, git, python 等直接可用的命令）
fn detect_tool(name: &str) -> Option<String> {
    get_command_output(name, &["--version"])
}

/// Python 版本检测
fn detect_python() -> Option<String> {
    get_command_output("python", &["--version"])
        .or_else(|| get_command_output("python3", &["--version"]))
        .map(|s| s.strip_prefix("Python ").unwrap_or(&s).to_string())
}

/// Git 版本检测
fn detect_git() -> Option<String> {
    get_command_output("git", &["--version"]).map(|s| {
        let ver = s.replace("git version ", "");
        let short = ver.split_whitespace().next().unwrap_or("");
        // "2.53.0.windows.2" → "2.53.0"
        short.split('.').take(3).collect::<Vec<_>>().join(".")
    })
}

/// npm 检测（npm 是 .cmd 文件，必须通过 cmd /C 执行）
fn detect_npm() -> Option<String> {
    get_command_output("npm", &["--version"])
}

/// VS Code 检测（code 是 .cmd 文件）
fn detect_vscode() -> Option<String> {
    // cmd /C code --version（PATH 中有 code.cmd 所在目录）
    if let Some(v) = get_command_output("code", &["--version"]) {
        return Some(v);
    }

    #[cfg(target_os = "windows")]
    {
        // 固定路径
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let code_cmd = format!(r"{}\Programs\Microsoft VS Code\bin\code.cmd", local);
            if std::path::Path::new(&code_cmd).exists() {
                log::debug!("[env_check] VS Code 固定路径: {}", code_cmd);
                // 通过 cmd /C 执行 .cmd 文件
                let path = get_shell_path();
                let mut c = Command::new("cmd");
                c.creation_flags(0x08000000);
                let result = c
                    .args(["/C", &code_cmd, "--version"])
                    .env("PATH", &path)
                    .output()
                    .ok()
                    .and_then(|o| {
                        if o.status.success() {
                            String::from_utf8(o.stdout)
                                .ok()
                                .map(|s| s.lines().next().unwrap_or("").trim().to_string())
                        } else {
                            None
                        }
                    })
                    .filter(|s| !s.is_empty());
                if result.is_some() {
                    return result;
                }
            }
        }
        None
    }

    #[cfg(not(target_os = "windows"))]
    {
        let candidates = [
            "/usr/local/bin/code",
            "/opt/homebrew/bin/code",
            "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code",
        ];
        for p in &candidates {
            if std::path::Path::new(p).exists() {
                if let Some(v) = get_command_output_at(p, &["--version"]) {
                    return Some(v);
                }
            }
        }
        None
    }
}

/// Chrome 检测（从注册表读版本号，最可靠）
fn detect_chrome() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        // 方法1：从 BLBeacon 注册表直接读版本号（不需要找 exe）
        let beacon_paths = [
            (HKEY_CURRENT_USER, r"SOFTWARE\Google\Chrome\BLBeacon"),
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\Google\Chrome\BLBeacon"),
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Google\Chrome\BLBeacon"),
        ];
        for (hive, path) in &beacon_paths {
            if let Ok(key) = RegKey::predef(*hive).open_subkey_with_flags(path, KEY_READ) {
                if let Ok(version) = key.get_value::<String, _>("version") {
                    if !version.is_empty() {
                        log::debug!("[env_check] Chrome BLBeacon 版本: {}", version);
                        return Some(version);
                    }
                }
            }
        }

        // 方法2：从 App Paths 读 exe 路径，执行 --version
        let app_paths = [
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\chrome.exe"),
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths\chrome.exe"),
            (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\chrome.exe"),
        ];
        for (hive, path) in &app_paths {
            if let Ok(key) = RegKey::predef(*hive).open_subkey_with_flags(path, KEY_READ) {
                if let Ok(exe_path) = key.get_value::<String, _>("") {
                    log::debug!("[env_check] Chrome App Paths: {}", exe_path);
                    if let Some(v) = get_command_output_at(&exe_path, &["--version"]) {
                        return Some(v.replace("Google Chrome ", "").replace("Chromium ", ""));
                    }
                }
            }
        }

        // 方法3：固定路径
        let fixed = [
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ];
        for p in &fixed {
            if std::path::Path::new(p).exists() {
                log::debug!("[env_check] Chrome 固定路径: {}", p);
                if let Some(v) = get_command_output_at(p, &["--version"]) {
                    return Some(v.replace("Google Chrome ", "").replace("Chromium ", ""));
                }
            }
        }

        None
    }

    #[cfg(target_os = "macos")]
    {
        let app_binary = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
        if std::path::Path::new(app_binary).exists() {
            return get_command_output_at(app_binary, &["--version"])
                .map(|s| s.replace("Google Chrome ", ""));
        }
        get_command_output("google-chrome", &["--version"])
            .map(|s| s.replace("Google Chrome ", "").replace("Chromium ", ""))
    }

    #[cfg(target_os = "linux")]
    {
        get_command_output("google-chrome", &["--version"])
            .or_else(|| get_command_output("google-chrome-stable", &["--version"]))
            .or_else(|| get_command_output("chromium-browser", &["--version"]))
            .or_else(|| get_command_output("chromium", &["--version"]))
            .map(|s| s.replace("Google Chrome ", "").replace("Chromium ", ""))
    }
}

/// Claude CLI 检测（可能是 .exe 或 .cmd，通过 cmd /C 执行）
fn detect_claude() -> Option<String> {
    get_command_output("claude", &["--version"])
        .map(|s| s.split_whitespace().next().unwrap_or(&s).to_string())
}

/// 检测所有环境工具的版本
#[tauri::command]
pub fn check_env() -> HashMap<String, Option<String>> {
    log::info!("[env_check] 开始环境检测...");
    let mut result = HashMap::new();

    result.insert("nodejs".to_string(), detect_tool("node"));
    result.insert("npm".to_string(), detect_npm());
    result.insert("python".to_string(), detect_python());
    result.insert("git".to_string(), detect_git());
    result.insert("vscode".to_string(), detect_vscode());
    result.insert("chrome".to_string(), detect_chrome());
    result.insert("claude".to_string(), detect_claude());

    log::info!("[env_check] 检测完成: {:?}", result);
    result
}

/// 检测单个工具的版本
#[tauri::command]
pub fn check_single_env(tool: String) -> Option<String> {
    log::info!("[env_check] 单项检测: {}", tool);
    match tool.as_str() {
        "nodejs" => detect_tool("node"),
        "npm" => detect_npm(),
        "python" => detect_python(),
        "git" => detect_git(),
        "vscode" => detect_vscode(),
        "chrome" => detect_chrome(),
        "claude" => detect_claude(),
        _ => None,
    }
}

/// 检测 npm 是否可用
#[tauri::command]
pub fn check_npm_available() -> bool {
    get_command_output("npm", &["config", "get", "registry"]).is_some()
}

/// 修复 npm registry
#[tauri::command]
pub fn fix_npm_registry() -> (bool, String) {
    let path = get_shell_path();

    #[cfg(target_os = "windows")]
    {
        let mut c = Command::new("cmd");
        c.creation_flags(0x08000000);
        match c
            .args(["/C", "npm", "config", "set", "registry", "https://registry.npmjs.org"])
            .env("PATH", &path)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    (true, "已修复 npm registry".to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    (false, format!("修复失败: {}", stderr))
                }
            }
            Err(e) => (false, format!("执行命令失败: {}", e)),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        match Command::new("npm")
            .args(["config", "set", "registry", "https://registry.npmjs.org"])
            .env("PATH", &path)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    (true, "已修复 npm registry".to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    (false, format!("修复失败: {}", stderr))
                }
            }
            Err(e) => (false, format!("执行命令失败: {}", e)),
        }
    }
}

/// 获取安装日志文件路径
fn get_install_log_path() -> std::path::PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("DevClaw");
    std::fs::create_dir_all(&dir).ok();
    dir.join("install.log")
}

/// 追加一条安装日志
fn append_install_log(tool: &str, success: bool, msg: &str) {
    use std::io::Write;
    let path = get_install_log_path();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let status = if success { "成功" } else { "失败" };
    let line = format!("[{}] {} - {} {}\n", now, tool, status, msg);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}

/// 读取安装日志
#[tauri::command]
pub fn read_install_log() -> String {
    let path = get_install_log_path();
    std::fs::read_to_string(&path).unwrap_or_else(|_| "暂无安装日志".to_string())
}

/// 静默执行一条命令（失败不中断）
fn run_cmd_silent(exe: &str, args: &[&str], path: &str) -> bool {
    let mut c = Command::new(exe);
    #[cfg(target_os = "windows")]
    c.creation_flags(0x08000000);
    c.args(args).env("PATH", path).output().map(|o| o.status.success()).unwrap_or(false)
}

/// 静默删除目录（失败不中断）
fn remove_dir_silent(p: &str) -> bool {
    let path = std::path::Path::new(p);
    if path.exists() {
        std::fs::remove_dir_all(path).is_ok()
    } else {
        true
    }
}

/// 使用 winget/npm 安装工具（先彻底卸载再安装，带进度事件）
/// 进度：0-30% 卸载阶段，30-100% 安装阶段
#[tauri::command]
pub fn install_tool(tool: String, window: tauri::Window) -> (bool, String) {
    let path = get_shell_path();

    #[cfg(target_os = "windows")]
    {
        use std::io::{BufRead, BufReader};
        use std::process::Stdio;
        use std::sync::{Arc, Mutex};
        use std::time::Duration;

        // ── 卸载阶段 (0% → 30%) ──────────────────────────────────────────
        let emit = |pct: u32| {
            let _ = window.emit("install_progress", serde_json::json!({
                "tool": &tool, "progress": pct, "done": false
            }));
        };

        let step = |label: &str, args: &[&str]| -> bool {
            log::info!("[uninstall] {} {:?}", label, args);
            run_cmd_silent(args[0], &args[1..], &path)
        };

        let uninstall_steps: Vec<(&str, Vec<&str>)> = match tool.as_str() {
            "nodejs" | "npm" => vec![
                ("winget node",   vec!["cmd", "/C", "winget", "uninstall", "OpenJS.NodeJS", "--silent"]),
                ("winget node lts",vec!["cmd", "/C", "winget", "uninstall", "OpenJS.NodeJS.LTS", "--silent"]),
                ("nvm uninstall", vec!["cmd", "/C", "nvm", "uninstall", "current"]),
                ("rm nodejs dir", vec!["__rm_dir__", r"C:\Program Files\nodejs"]),
                ("rm npm",        vec!["__rm_dir__", r"__APPDATA__\npm"]),
                ("rm npm-cache",  vec!["__rm_dir__", r"__APPDATA__\npm-cache"]),
            ],
            "python" => vec![
                ("winget python",   vec!["cmd", "/C", "winget", "uninstall", "Python.Python.3", "--silent"]),
                ("winget python313",vec!["cmd", "/C", "winget", "uninstall", "Python.Python.3.13", "--silent"]),
                ("winget python312",vec!["cmd", "/C", "winget", "uninstall", "Python.Python.3.12", "--silent"]),
                ("rm python dir",   vec!["__rm_dir__", r"__LOCALAPPDATA__\Programs\Python"]),
                ("rm appdata python",vec!["__rm_dir__", r"__APPDATA__\Python"]),
            ],
            "git" => vec![
                ("winget git",   vec!["cmd", "/C", "winget", "uninstall", "Git.Git", "--silent"]),
                ("rm git dir",   vec!["__rm_dir__", r"C:\Program Files\Git"]),
            ],
            "vscode" => vec![
                ("winget vscode",   vec!["cmd", "/C", "winget", "uninstall", "Microsoft.VisualStudioCode", "--silent"]),
                ("rm vscode dir",   vec!["__rm_dir__", r"__LOCALAPPDATA__\Programs\Microsoft VS Code"]),
            ],
            "chrome" => vec![
                ("winget chrome",   vec!["cmd", "/C", "winget", "uninstall", "Google.Chrome", "--silent"]),
                ("rm chrome dir",   vec!["__rm_dir__", r"__LOCALAPPDATA__\Google\Chrome"]),
            ],
            "claude" => vec![
                ("npm uninstall claude", vec!["cmd", "/C", "npm", "uninstall", "-g", "@anthropic-ai/claude-code"]),
                ("rm claude bin",        vec!["__rm_dir__", r"__USERPROFILE__\.local\bin"]),
                ("rm claude config",     vec!["__rm_dir__", r"__USERPROFILE__\.claude"]),
            ],
            _ => vec![],
        }
        ;

        let total = uninstall_steps.len() as u32;
        for (i, (label, args)) in uninstall_steps.iter().enumerate() {
            let pct = ((i as u32 + 1) * 30) / total.max(1);
            emit(pct);
            if args[0] == "__rm_dir__" {
                let p = args[1]
                    .replace("__APPDATA__", &std::env::var("APPDATA").unwrap_or_default())
                    .replace("__LOCALAPPDATA__", &std::env::var("LOCALAPPDATA").unwrap_or_default())
                    .replace("__USERPROFILE__", &std::env::var("USERPROFILE").unwrap_or_default());
                let ok = remove_dir_silent(&p);
                log::info!("[uninstall] rm {} => {}", p, ok);
            } else {
                let ok = step(label, args);
                log::info!("[uninstall] {} => {}", label, ok);
            }
        }

        // 卸载完成后等待 2 秒
        std::thread::sleep(Duration::from_secs(2));

        // ── 安装阶段 (30% → 100%) ────────────────────────────────────────
        let full_cmd = match tool.as_str() {
            "nodejs" | "npm" => "winget install OpenJS.NodeJS --accept-package-agreements --accept-source-agreements",
            "python" => "winget install Python.Python.3 --accept-package-agreements --accept-source-agreements",
            "git" => "winget install Git.Git --accept-package-agreements --accept-source-agreements",
            "vscode" => "winget install Microsoft.VisualStudioCode --accept-package-agreements --accept-source-agreements",
            "chrome" => "winget install Google.Chrome --accept-package-agreements --accept-source-agreements",
            "claude" => "npm install -g @anthropic-ai/claude-code",
            _ => return (false, "不支持的工具".to_string()),
        };

        let mut c = Command::new("cmd");
        c.creation_flags(0x08000000);
        let child = c
            .args(["/C", full_cmd])
            .env("PATH", &path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(e) => return (false, format!("执行安装命令失败: {}", e)),
        };

        let re = regex::Regex::new(r"(\d{1,3})%").unwrap();
        let last_pct = Arc::new(Mutex::new(0u32));
        let tool_clone = tool.clone();
        let last_clone = last_pct.clone();
        let window_clone = window.clone();

        if let Some(stdout) = child.stdout.take() {
            let re = re.clone();
            let tool = tool_clone.clone();
            let last = last_clone.clone();
            let win = window_clone.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    let line = match line { Ok(l) => l, Err(_) => continue };
                    log::debug!("[install] {}", line);
                    if let Some(cap) = re.captures(&line) {
                        if let Ok(pct) = cap[1].parse::<u32>() {
                            let mut last = last.lock().unwrap();
                            if pct > *last {
                                *last = pct;
                                let mapped = 30 + pct * 70 / 100;
                                let _ = win.emit("install_progress", serde_json::json!({
                                    "tool": &tool, "progress": mapped, "done": false
                                }));
                            }
                        }
                    }
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().flatten() {
                    log::debug!("[install stderr] {}", line);
                }
            });
        }

        // winget 无百分比时自动递增
        let tool_timer = tool_clone.clone();
        let last_timer = last_pct.clone();
        let win_timer = window_clone.clone();
        let running = Arc::new(Mutex::new(true));
        let running_timer = running.clone();
        let timer = std::thread::spawn(move || {
            while *running_timer.lock().unwrap() {
                std::thread::sleep(Duration::from_millis(500));
                let mut last = last_timer.lock().unwrap();
                if *last == 0 && *last < 90 {
                    *last += 15;
                    if *last > 90 { *last = 90; }
                    let mapped = 30 + *last * 70 / 100;
                    let _ = win_timer.emit("install_progress", serde_json::json!({
                        "tool": &tool_timer, "progress": mapped, "done": false
                    }));
                }
            }
        });

        let status = child.wait();
        *running.lock().unwrap() = false;
        let _ = timer.join();

        let success = status.map(|s| s.success()).unwrap_or(false);
        let msg = if success { "安装成功".to_string() } else { "安装失败".to_string() };

        append_install_log(&tool, success, &msg);

        let _ = window.emit("install_progress", serde_json::json!({
            "tool": &tool, "progress": 100, "done": true
        }));

        (success, msg)
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::time::Duration;

        // ── 卸载阶段 (0% → 30%) ──────────────────────────────────────────
        let emit = |pct: u32| {
            let _ = window.emit("install_progress", serde_json::json!({
                "tool": &tool, "progress": pct, "done": false
            }));
        };

        let home = std::env::var("HOME").unwrap_or_default();

        // 类型: ("__cmd__", cmd) 执行 shell 命令 | ("__rm__", path) 删除目录
        let uninstall_steps: Vec<(&str, &str)> = match tool.as_str() {
            "nodejs" | "npm" => vec![
                ("__cmd__", "brew uninstall node 2>/dev/null"),
                ("__cmd__", "brew uninstall node@22 2>/dev/null; brew uninstall node@20 2>/dev/null; brew uninstall node@18 2>/dev/null"),
                ("__cmd__", "test -s ~/.nvm/nvm.sh && . ~/.nvm/nvm.sh 2>/dev/null && nvm uninstall $(nvm current) 2>/dev/null || true"),
                ("__rm__",  "/usr/local/bin/node"),
                ("__rm__",  "/opt/homebrew/bin/node"),
                ("__rm__",  "/usr/local/bin/npm"),
                ("__rm__",  "/opt/homebrew/bin/npm"),
                ("__rm_h__", "~/.npm"),
                ("__rm_h__", "~/.npm-cache"),
            ],
            "python" => vec![
                ("__cmd__", "brew uninstall python 2>/dev/null"),
                ("__cmd__", "brew uninstall python3 2>/dev/null"),
                ("__rm__",  "/usr/local/bin/python3"),
                ("__rm__",  "/opt/homebrew/bin/python3"),
                ("__rm__",  "/usr/local/bin/pip3"),
                ("__rm__",  "/opt/homebrew/bin/pip3"),
            ],
            "git" => vec![
                ("__cmd__", "brew uninstall git 2>/dev/null"),
                ("__rm__",  "/usr/local/bin/git"),
                ("__rm__",  "/opt/homebrew/bin/git"),
            ],
            "vscode" => vec![
                ("__rm__",  "/Applications/Visual Studio Code.app"),
                ("__rm_h__", "~/Library/Application Support/Code"),
                ("__rm_h__", "~/.vscode"),
            ],
            "chrome" => vec![
                ("__rm__",  "/Applications/Google Chrome.app"),
                ("__rm_h__", "~/Library/Application Support/Google/Chrome"),
            ],
            "claude" => vec![
                ("__cmd__", "npm uninstall -g @anthropic-ai/claude-code 2>/dev/null"),
                ("__rm_h__", "~/.local/bin/claude"),
                ("__rm_h__", "~/.claude"),
            ],
            _ => vec![],
        };

        let total = uninstall_steps.len() as u32;
        for (i, (kind, val)) in uninstall_steps.iter().enumerate() {
            let pct = ((i as u32 + 1) * 30) / total.max(1);
            emit(pct);
            match *kind {
                "__cmd__" => {
                    let ok = run_cmd_silent("/bin/zsh", &["-c", val], &path);
                    log::info!("[uninstall] cmd '{}' => {}", val, ok);
                }
                "__rm__" => {
                    let ok = remove_dir_silent(val);
                    log::info!("[uninstall] rm '{}' => {}", val, ok);
                }
                "__rm_h__" => {
                    let expanded = val.replace("~", &home);
                    let ok = remove_dir_silent(&expanded);
                    log::info!("[uninstall] rm '{}' => {}", expanded, ok);
                }
                _ => {}
            }
        }

        std::thread::sleep(Duration::from_secs(2));

        // ── 安装阶段 (30% → 100%) ────────────────────────────────────────
        emit(35);
        let (cmd, args) = match tool.as_str() {
            "nodejs" | "npm" => ("/bin/zsh", vec!["-c", "brew install node"]),
            "python" => ("/bin/zsh", vec!["-c", "brew install python3"]),
            "git" => ("/bin/zsh", vec!["-c", "brew install git"]),
            "vscode" => ("/bin/zsh", vec!["-c", "brew install --cask visual-studio-code"]),
            "chrome" => ("/bin/zsh", vec!["-c", "brew install --cask google-chrome"]),
            "claude" => ("/bin/zsh", vec!["-c", "npm install -g @anthropic-ai/claude-code"]),
            _ => return (false, "不支持的工具".to_string()),
        };

        let result = run_cmd_silent(cmd, &args, &path);
        emit(100);

        let (success, msg) = if result {
            (true, "安装成功".to_string())
        } else {
            (false, "安装失败".to_string())
        };
        append_install_log(&tool, success, &msg);

        let _ = window.emit("install_progress", serde_json::json!({
            "tool": &tool, "progress": 100, "done": true
        }));

        (success, msg)
    }
}

/// 调试命令
#[cfg(target_os = "windows")]
#[tauri::command]
pub fn debug_env() -> String {
    let mut out = String::new();
    let mut log_line = |label: &str, msg: &str| {
        let line = format!("[{}] {}", label, msg);
        eprintln!("{}", line);
        out.push_str(&line);
        out.push('\n');
    };

    log_line("=== debug_env ===", "");

    let proc_path = std::env::var("PATH").unwrap_or_else(|e| format!("<读取失败: {}>", e));
    log_line("进程PATH", &proc_path);

    let shell_path = get_shell_path();
    log_line("shellPATH", &shell_path);

    // which crate
    match which::which("node") {
        Ok(p) => log_line("which(node)", &p.display().to_string()),
        Err(e) => log_line("which(node)", &format!("未找到: {}", e)),
    }

    // node
    log_line("--- node ---", "");
    match { let mut c = Command::new("cmd"); c.creation_flags(0x08000000); c }.args(["/C", "node", "--version"]).env("PATH", &shell_path).output() {
        Ok(o) => log_line("cmd /C node --version", &format!("exit={:?} stdout=[{}] stderr=[{}]", o.status.code(), String::from_utf8_lossy(&o.stdout).trim(), String::from_utf8_lossy(&o.stderr).trim())),
        Err(e) => log_line("cmd /C node --version", &format!("错误: {}", e)),
    }

    // npm
    log_line("--- npm ---", "");
    match { let mut c = Command::new("cmd"); c.creation_flags(0x08000000); c }.args(["/C", "npm", "--version"]).env("PATH", &shell_path).output() {
        Ok(o) => log_line("cmd /C npm --version", &format!("exit={:?} stdout=[{}] stderr=[{}]", o.status.code(), String::from_utf8_lossy(&o.stdout).trim(), String::from_utf8_lossy(&o.stderr).trim())),
        Err(e) => log_line("cmd /C npm --version", &format!("错误: {}", e)),
    }

    // code
    log_line("--- code ---", "");
    match { let mut c = Command::new("cmd"); c.creation_flags(0x08000000); c }.args(["/C", "code", "--version"]).env("PATH", &shell_path).output() {
        Ok(o) => log_line("cmd /C code --version", &format!("exit={:?} stdout=[{}] stderr=[{}]", o.status.code(), String::from_utf8_lossy(&o.stdout).trim(), String::from_utf8_lossy(&o.stderr).trim())),
        Err(e) => log_line("cmd /C code --version", &format!("错误: {}", e)),
    }

    // git
    log_line("--- git ---", "");
    match { let mut c = Command::new("cmd"); c.creation_flags(0x08000000); c }.args(["/C", "git", "--version"]).env("PATH", &shell_path).output() {
        Ok(o) => log_line("cmd /C git --version", &format!("exit={:?} stdout=[{}] stderr=[{}]", o.status.code(), String::from_utf8_lossy(&o.stdout).trim(), String::from_utf8_lossy(&o.stderr).trim())),
        Err(e) => log_line("cmd /C git --version", &format!("错误: {}", e)),
    }

    // claude
    log_line("--- claude ---", "");
    match { let mut c = Command::new("cmd"); c.creation_flags(0x08000000); c }.args(["/C", "claude", "--version"]).env("PATH", &shell_path).output() {
        Ok(o) => log_line("cmd /C claude --version", &format!("exit={:?} stdout=[{}] stderr=[{}]", o.status.code(), String::from_utf8_lossy(&o.stdout).trim(), String::from_utf8_lossy(&o.stderr).trim())),
        Err(e) => log_line("cmd /C claude --version", &format!("错误: {}", e)),
    }

    // chrome — 注册表检测
    log_line("--- chrome (注册表) ---", "");
    {
        use winreg::enums::*;
        use winreg::RegKey;

        // BLBeacon 版本号
        let beacon_paths = [
            (HKEY_CURRENT_USER, r"SOFTWARE\Google\Chrome\BLBeacon"),
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\Google\Chrome\BLBeacon"),
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Google\Chrome\BLBeacon"),
        ];
        for (hive, path) in &beacon_paths {
            match RegKey::predef(*hive).open_subkey_with_flags(path, KEY_READ) {
                Ok(key) => match key.get_value::<String, _>("version") {
                    Ok(v) => log_line("BLBeacon version", &format!("{} => {}", path, v)),
                    Err(e) => log_line("BLBeacon version", &format!("{} => 读取失败: {}", path, e)),
                },
                Err(e) => log_line("BLBeacon key", &format!("{} => 打开失败: {}", path, e)),
            }
        }

        // App Paths exe 路径
        let app_paths = [
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\chrome.exe"),
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths\chrome.exe"),
            (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\chrome.exe"),
        ];
        for (hive, path) in &app_paths {
            match RegKey::predef(*hive).open_subkey_with_flags(path, KEY_READ) {
                Ok(key) => match key.get_value::<String, _>("") {
                    Ok(exe) => {
                        log_line("App Paths exe", &format!("{} => {}", path, exe));
                        match { let mut c = Command::new(&exe); c.creation_flags(0x08000000); c }.arg("--version").env("PATH", &shell_path).output() {
                            Ok(o) => log_line("chrome --version", &format!("exit={:?} stdout=[{}] stderr=[{}]", o.status.code(), String::from_utf8_lossy(&o.stdout).trim(), String::from_utf8_lossy(&o.stderr).trim())),
                            Err(e) => log_line("chrome --version", &format!("错误: {}", e)),
                        }
                    },
                    Err(e) => log_line("App Paths exe", &format!("{} => 读取失败: {}", path, e)),
                },
                Err(e) => log_line("App Paths key", &format!("{} => 打开失败: {}", path, e)),
            }
        }
    }

    log_line("=== debug_env 结束 ===", "");
    out
}

/// 调试命令
#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn debug_env() -> String {
    "debug_env: 仅支持 Windows 平台".to_string()
}
