use std::collections::HashMap;
use std::process::Command;

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
        let result = Command::new("cmd")
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
    let result = Command::new(exe_path)
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
        s.replace("git version ", "")
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
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
                let result = Command::new("cmd")
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

/// 使用 winget/npm 安装工具
#[tauri::command]
pub fn install_tool(tool: String) -> (bool, String) {
    let path = get_shell_path();

    #[cfg(target_os = "windows")]
    {
        let full_cmd = match tool.as_str() {
            "nodejs" | "npm" => "winget install OpenJS.NodeJS --accept-package-agreements --accept-source-agreements",
            "python" => "winget install Python.Python.3 --accept-package-agreements --accept-source-agreements",
            "git" => "winget install Git.Git --accept-package-agreements --accept-source-agreements",
            "vscode" => "winget install Microsoft.VisualStudioCode --accept-package-agreements --accept-source-agreements",
            "chrome" => "winget install Google.Chrome --accept-package-agreements --accept-source-agreements",
            "claude" => "npm install -g @anthropic-ai/claude-code",
            _ => return (false, "不支持的工具".to_string()),
        };

        match Command::new("cmd")
            .args(["/C", full_cmd])
            .env("PATH", &path)
            .output()
        {
            Ok(output) => {
                let success = output.status.success();
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let msg = if success {
                    format!("安装成功\n{}", stdout.trim())
                } else {
                    format!("安装失败\n{}", if stderr.trim().is_empty() { stdout.trim() } else { stderr.trim() })
                };
                (success, msg)
            }
            Err(e) => (false, format!("执行安装命令失败: {}", e)),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let (cmd, args) = match tool.as_str() {
            "nodejs" | "npm" => ("brew", vec!["install", "node"]),
            "python" => ("brew", vec!["install", "python3"]),
            "git" => ("brew", vec!["install", "git"]),
            "vscode" => ("brew", vec!["install", "--cask", "visual-studio-code"]),
            "chrome" => ("brew", vec!["install", "--cask", "google-chrome"]),
            "claude" => ("npm", vec!["install", "-g", "@anthropic-ai/claude-code"]),
            _ => return (false, "不支持的工具".to_string()),
        };

        match Command::new(cmd).args(&args).env("PATH", &path).output() {
            Ok(output) => {
                let success = output.status.success();
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let msg = if success {
                    format!("安装成功\n{}", stdout.trim())
                } else {
                    format!("安装失败\n{}", if stderr.trim().is_empty() { stdout.trim() } else { stderr.trim() })
                };
                (success, msg)
            }
            Err(e) => (false, format!("执行安装命令失败: {}", e)),
        }
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
    match Command::new("cmd").args(["/C", "node", "--version"]).env("PATH", &shell_path).output() {
        Ok(o) => log_line("cmd /C node --version", &format!("exit={:?} stdout=[{}] stderr=[{}]", o.status.code(), String::from_utf8_lossy(&o.stdout).trim(), String::from_utf8_lossy(&o.stderr).trim())),
        Err(e) => log_line("cmd /C node --version", &format!("错误: {}", e)),
    }

    // npm
    log_line("--- npm ---", "");
    match Command::new("cmd").args(["/C", "npm", "--version"]).env("PATH", &shell_path).output() {
        Ok(o) => log_line("cmd /C npm --version", &format!("exit={:?} stdout=[{}] stderr=[{}]", o.status.code(), String::from_utf8_lossy(&o.stdout).trim(), String::from_utf8_lossy(&o.stderr).trim())),
        Err(e) => log_line("cmd /C npm --version", &format!("错误: {}", e)),
    }

    // code
    log_line("--- code ---", "");
    match Command::new("cmd").args(["/C", "code", "--version"]).env("PATH", &shell_path).output() {
        Ok(o) => log_line("cmd /C code --version", &format!("exit={:?} stdout=[{}] stderr=[{}]", o.status.code(), String::from_utf8_lossy(&o.stdout).trim(), String::from_utf8_lossy(&o.stderr).trim())),
        Err(e) => log_line("cmd /C code --version", &format!("错误: {}", e)),
    }

    // git
    log_line("--- git ---", "");
    match Command::new("cmd").args(["/C", "git", "--version"]).env("PATH", &shell_path).output() {
        Ok(o) => log_line("cmd /C git --version", &format!("exit={:?} stdout=[{}] stderr=[{}]", o.status.code(), String::from_utf8_lossy(&o.stdout).trim(), String::from_utf8_lossy(&o.stderr).trim())),
        Err(e) => log_line("cmd /C git --version", &format!("错误: {}", e)),
    }

    // claude
    log_line("--- claude ---", "");
    match Command::new("cmd").args(["/C", "claude", "--version"]).env("PATH", &shell_path).output() {
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
                        match Command::new(&exe).arg("--version").env("PATH", &shell_path).output() {
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
