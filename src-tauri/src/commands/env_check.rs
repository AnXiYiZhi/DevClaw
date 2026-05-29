use std::collections::HashMap;
use std::process::Command;
use tauri::Emitter;
use tauri_plugin_opener::OpenerExt;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// 获取 shell 的完整 PATH
fn get_shell_path() -> String {
    let current = std::env::var("PATH").unwrap_or_default();

    #[cfg(not(target_os = "windows"))]
    {
        // 多种方式获取 PATH，按优先级尝试
        let shell_path = get_shell_path_from_login_shell()
            .or_else(get_shell_path_from_env_file);

        if let Some(sp) = shell_path {
            if !sp.is_empty() {
                // 去重合并：shell PATH 优先，追加 current 中不重复的部分
                let mut seen = std::collections::HashSet::new();
                let mut merged = String::new();
                for p in sp.split(':').chain(current.split(':')) {
                    if !p.is_empty() && seen.insert(p.to_string()) {
                        if !merged.is_empty() {
                            merged.push(':');
                        }
                        merged.push_str(p);
                    }
                }
                return merged;
            }
        }
    }

    current
}

/// 从 login shell 获取 PATH
#[cfg(not(target_os = "windows"))]
fn get_shell_path_from_login_shell() -> Option<String> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let output = Command::new(&shell)
        .args(["-l", "-c", "echo $PATH"])
        .output()
        .ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            log::debug!("[env_check] 从 login shell ({}) 获取 PATH 成功", shell);
            return Some(path);
        }
    }
    // 回退到 zsh
    if shell != "/bin/zsh" {
        let output = Command::new("/bin/zsh")
            .args(["-l", "-c", "echo $PATH"])
            .output()
            .ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                log::debug!("[env_check] 从 /bin/zsh 回退获取 PATH 成功");
                return Some(path);
            }
        }
    }
    None
}

/// 从 shell 配置文件中提取 PATH 设置
#[cfg(not(target_os = "windows"))]
fn get_shell_path_from_env_file() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let config_files = [
        format!("{}/.zshrc", home),
        format!("{}/.zprofile", home),
        format!("{}/.bashrc", home),
        format!("{}/.bash_profile", home),
        format!("{}/.profile", home),
    ];

    let mut extra_paths = Vec::new();
    for file in &config_files {
        if let Ok(content) = std::fs::read_to_string(file) {
            for line in content.lines() {
                let trimmed = line.trim();
                // 匹配 export PATH=... 或 PATH=...
                if let Some(rest) = trimmed.strip_prefix("export ").or(Some(trimmed)) {
                    let rest = rest.trim();
                    if rest.starts_with("PATH") && rest.contains('=') {
                        // 提取 PATH 值
                        if let Some(eq_pos) = rest.find('=') {
                            let val = rest[eq_pos + 1..].trim();
                            // 处理 $PATH 引用，提取新增路径
                            for part in val.replace("$PATH", "").replace("${PATH}", "").split(':') {
                                let p = part.trim().trim_matches('"').trim_matches('\'');
                                if !p.is_empty() && p.starts_with('/') {
                                    extra_paths.push(p.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if extra_paths.is_empty() {
        return None;
    }
    log::debug!("[env_check] 从 shell 配置文件提取到额外 PATH: {:?}", extra_paths);
    let current = std::env::var("PATH").unwrap_or_default();
    let mut merged = extra_paths.join(":");
    if !current.is_empty() {
        merged.push(':');
        merged.push_str(&current);
    }
    Some(merged)
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
pub async fn check_env() -> HashMap<String, Option<String>> {
    tauri::async_runtime::spawn_blocking(|| {
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
    })
    .await
    .unwrap_or_default()
}

/// 检测单个工具的版本
#[tauri::command]
pub async fn check_single_env(tool: String) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || {
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
    })
    .await
    .unwrap_or(None)
}

/// 检测 npm 是否可用
#[tauri::command]
pub async fn check_npm_available() -> bool {
    tauri::async_runtime::spawn_blocking(|| {
        get_command_output("npm", &["config", "get", "registry"]).is_some()
    })
    .await
    .unwrap_or(false)
}

/// 修复 npm registry
#[tauri::command]
pub async fn fix_npm_registry() -> (bool, String) {
    tauri::async_runtime::spawn_blocking(|| {
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
    })
    .await
    .unwrap_or((false, "修复线程异常".to_string()))
}

/// 追加一条安装日志
fn append_install_log(tool: &str, success: bool, msg: &str) {
    write_install_log(tool, success, msg);
}

/// 静默执行一条命令（失败不中断）
fn run_cmd_silent(exe: &str, args: &[&str], path: &str) -> bool {
    let mut c = Command::new(exe);
    #[cfg(target_os = "windows")]
    c.creation_flags(0x08000000);
    c.args(args).env("PATH", path).output().map(|o| o.status.success()).unwrap_or(false)
}

/// 执行命令并捕获输出，写入详细日志
fn run_cmd_logged(tool: &str, label: &str, exe: &str, args: &[&str], path: &str) -> (bool, String, String) {
    let mut c = Command::new(exe);
    #[cfg(target_os = "windows")]
    c.creation_flags(0x08000000);
    let output = c.args(args).env("PATH", path).output();
    let cmd_str = format!("{} {}", exe, args.join(" "));
    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let success = o.status.success();
            let code = o.status.code().unwrap_or(-1);
            let mut log_entry = format!(
                "[{}] {} | 退出码: {}\n  命令: {}\n  PATH: {}",
                label, if success { "成功" } else { "失败" }, code, cmd_str, path
            );
            if !stdout.trim().is_empty() {
                log_entry.push_str(&format!("\n  stdout:\n{}", stdout.trim()));
            }
            if !stderr.trim().is_empty() {
                log_entry.push_str(&format!("\n  stderr:\n{}", stderr.trim()));
            }
            append_install_log(tool, success, &log_entry);
            (success, stdout, stderr)
        }
        Err(e) => {
            append_install_log(tool, false, &format!(
                "[{}] 执行失败\n  命令: {}\n  PATH: {}\n  错误: {}",
                label, cmd_str, path, e
            ));
            (false, String::new(), format!("执行失败: {}", e))
        }
    }
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

/// 检测 node 是否由 nvm 管理（Windows）
#[cfg(target_os = "windows")]
fn is_node_managed_by_nvm() -> bool {
    which::which("node")
        .map(|p| {
            let path_str = p.to_string_lossy().to_lowercase();
            path_str.contains("nvm")
        })
        .unwrap_or(false)
}

/// 检测 node 是否由 nvm 管理（macOS/Linux）
#[cfg(not(target_os = "windows"))]
fn is_node_managed_by_nvm() -> bool {
    which::which("node")
        .map(|p| {
            let path_str = p.to_string_lossy().to_lowercase();
            path_str.contains(".nvm") || path_str.contains("nvm")
        })
        .unwrap_or(false)
}

/// 检测 node 是否由 brew 管理（仅 macOS 有意义）
#[cfg(not(target_os = "windows"))]
fn is_node_managed_by_brew() -> bool {
    #[cfg(target_os = "macos")]
    {
        which::which("node")
            .map(|p| {
                let path_str = p.to_string_lossy();
                path_str.starts_with("/opt/homebrew/") || path_str.starts_with("/usr/local/Cellar/")
                    || path_str.starts_with("/usr/local/opt/")
            })
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// 使用 winget/npm 安装工具（带进度事件）
/// uninstall_first=true: 先卸载再安装（重装场景）
/// uninstall_first=false: 直接安装（首次安装场景）
/// 进度：0-30% 卸载阶段（仅 uninstall_first=true），30-100% 安装阶段
#[tauri::command]
pub async fn install_tool(tool: String, uninstall_first: bool, window: tauri::Window) -> (bool, String) {
    append_install_log(&tool, true, &format!("开始安装 (uninstall_first={})", uninstall_first));

    let result = tauri::async_runtime::spawn_blocking({
        let tool = tool.clone();
        let window = window.clone();
        move || install_tool_inner(tool, uninstall_first, window)
    })
    .await
    .unwrap_or_else(|e| (false, format!("安装线程异常: {}", e)));

    result
}

fn install_tool_inner(tool: String, uninstall_first: bool, window: tauri::Window) -> (bool, String) {
    let path = get_shell_path();

    #[cfg(target_os = "windows")]
    {
        use std::io::{BufRead, BufReader};
        use std::process::Stdio;
        use std::sync::{Arc, Mutex};
        use std::time::Duration;

        let emit = |pct: u32| {
            let _ = window.emit("install_progress", serde_json::json!({
                "tool": &tool, "progress": pct, "done": false
            }));
        };

        // ── 卸载阶段 (0% → 30%)，仅 uninstall_first=true ────────────────
        if uninstall_first {
            let uninstall_steps: Vec<(&str, Vec<&str>)> = match tool.as_str() {
                "nodejs" | "npm" => {
                    let mut steps = Vec::new();
                    if is_node_managed_by_nvm() {
                        // nvm 管理的 node：用 nvm uninstall 卸载当前版本
                        log::info!("[uninstall] 检测到 nvm 管理的 node，使用 nvm uninstall");
                        steps.push(("nvm uninstall current", vec!["cmd", "/C", "nvm", "uninstall", "current"]));
                    } else {
                        // 非 nvm 管理：用 winget 卸载
                        steps.push(("winget node",   vec!["cmd", "/C", "winget", "uninstall", "OpenJS.NodeJS", "--silent"]));
                        steps.push(("winget node lts",vec!["cmd", "/C", "winget", "uninstall", "OpenJS.NodeJS.LTS", "--silent"]));
                    }
                    steps.push(("rm nodejs dir", vec!["__rm_dir__", r"C:\Program Files\nodejs"]));
                    steps.push(("rm npm",        vec!["__rm_dir__", r"__APPDATA__\npm"]));
                    steps.push(("rm npm-cache",  vec!["__rm_dir__", r"__APPDATA__\npm-cache"]));
                    steps
                }
            "python" => vec![
                ("winget python313",vec!["cmd", "/C", "winget", "uninstall", "Python.Python.3.13", "--silent"]),
                ("winget python312",vec!["cmd", "/C", "winget", "uninstall", "Python.Python.3.12", "--silent"]),
                ("winget python311",vec!["cmd", "/C", "winget", "uninstall", "Python.Python.3.11", "--silent"]),
                ("winget python310",vec!["cmd", "/C", "winget", "uninstall", "Python.Python.3.10", "--silent"]),
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
                ("kill chrome",     vec!["cmd", "/C", "taskkill", "/F", "/IM", "chrome.exe"]),
                ("winget chrome",   vec!["cmd", "/C", "winget", "uninstall", "Google.Chrome", "--silent"]),
                ("rm chrome dir",   vec!["__rm_dir__", r"__LOCALAPPDATA__\Google\Chrome"]),
            ],
            "claude" => {
                let mut steps = Vec::new();
                // 只有 npm 可用时才执行 npm uninstall
                if which::which("npm").is_ok() {
                    steps.push(("npm uninstall claude", vec!["cmd", "/C", "npm", "uninstall", "-g", "@anthropic-ai/claude-code"]));
                }
                // 只删除 claude 可执行文件，不删除整个目录
                steps.push(("rm claude exe", vec!["__rm_file__", r"__USERPROFILE__\.local\bin\claude.exe"]));
                steps.push(("rm claude cmd", vec!["__rm_file__", r"__USERPROFILE__\.local\bin\claude"]));
                steps
            },
            _ => vec![],
        }
        ;

        let total = uninstall_steps.len() as u32;
        for (i, (label, args)) in uninstall_steps.iter().enumerate() {
            let pct = ((i as u32 + 1) * 30) / total.max(1);
            emit(pct);
            let p = args[1]
                .replace("__APPDATA__", &std::env::var("APPDATA").unwrap_or_default())
                .replace("__LOCALAPPDATA__", &std::env::var("LOCALAPPDATA").unwrap_or_default())
                .replace("__USERPROFILE__", &std::env::var("USERPROFILE").unwrap_or_default());
            if args[0] == "__rm_dir__" {
                let exists = std::path::Path::new(&p).exists();
                let ok = remove_dir_silent(&p);
                append_install_log(&tool, ok, &format!("[卸载:{}] 删除目录 '{}' (存在: {})", label, p, exists));
            } else if args[0] == "__rm_file__" {
                let path = std::path::Path::new(&p);
                let exists = path.exists();
                let ok = if exists { std::fs::remove_file(path).is_ok() } else { true };
                append_install_log(&tool, ok, &format!("[卸载:{}] 删除文件 '{}' (存在: {})", label, p, exists));
            } else {
                let _ = run_cmd_logged(&tool, &format!("卸载:{}", label), args[0], &args[1..], &path);
            }
        }

        // 卸载完成后等待 2 秒
        std::thread::sleep(Duration::from_secs(2));
        } // end if uninstall_first

        // ── 安装阶段 (30% → 100%) ────────────────────────────────────────
        let full_cmd = match tool.as_str() {
            "nodejs" | "npm" => "winget install OpenJS.NodeJS --force --accept-package-agreements --accept-source-agreements",
            "python" => "winget install Python.Python.3.13 --force --accept-package-agreements --accept-source-agreements",
            "git" => "winget install Git.Git --force --accept-package-agreements --accept-source-agreements",
            "vscode" => "winget install Microsoft.VisualStudioCode --force --accept-package-agreements --accept-source-agreements",
            "chrome" => "winget install Google.Chrome --force --accept-package-agreements --accept-source-agreements",
            "claude" => "npm install -g @anthropic-ai/claude-code",
            _ => return (false, "不支持的工具".to_string()),
        };

        append_install_log(&tool, true, &format!("[安装] 执行: {}", full_cmd));

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
            Err(e) => {
                let err = format!("执行安装命令失败: {}", e);
                append_install_log(&tool, false, &err);
                return (false, err);
            }
        };

        let re = regex::Regex::new(r"(\d{1,3})%").unwrap();
        let last_pct = Arc::new(Mutex::new(0u32));
        let tool_clone = tool.clone();
        let last_clone = last_pct.clone();
        let window_clone = window.clone();

        // 收集 stdout 输出用于日志
        let stdout_log = Arc::new(Mutex::new(String::new()));
        let stdout_log_clone = stdout_log.clone();

        if let Some(stdout) = child.stdout.take() {
            let re = re.clone();
            let tool = tool_clone.clone();
            let last = last_clone.clone();
            let win = window_clone.clone();
            let log_buf = stdout_log_clone;
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    let line = match line { Ok(l) => l, Err(_) => continue };
                    log::debug!("[install] {}", line);
                    log_buf.lock().unwrap().push_str(&line);
                    log_buf.lock().unwrap().push('\n');
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

        // 收集 stderr 输出用于日志
        let stderr_log = Arc::new(Mutex::new(String::new()));
        let stderr_log_clone = stderr_log.clone();

        if let Some(stderr) = child.stderr.take() {
            let log_buf = stderr_log_clone;
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().flatten() {
                    log::debug!("[install stderr] {}", line);
                    log_buf.lock().unwrap().push_str(&line);
                    log_buf.lock().unwrap().push('\n');
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
                if *last < 90 {
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

        let success = status.as_ref().map(|s| s.success()).unwrap_or(false);
        let exit_code = status.ok().and_then(|s| s.code()).unwrap_or(-1);
        let stdout_content = stdout_log.lock().unwrap().clone();
        let stderr_content = stderr_log.lock().unwrap().clone();

        let msg = if success {
            "安装成功".to_string()
        } else {
            format!("安装失败 (退出码: {})", exit_code)
        };
        append_install_log(&tool, success, &msg);

        // 记录详细输出
        if !stdout_content.trim().is_empty() {
            append_install_log(&tool, true, &format!("[安装:stdout]\n{}", stdout_content.trim()));
        }
        if !stderr_content.trim().is_empty() {
            append_install_log(&tool, false, &format!("[安装:stderr]\n{}", stderr_content.trim()));
        }

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

        // ── 卸载阶段 (0% → 30%)，仅 uninstall_first=true ────────────────
        if uninstall_first {
        // 类型: ("__cmd__", cmd) 执行 shell 命令 | ("__rm__", path) 删除目录
        let uninstall_steps: Vec<(&str, &str)> = match tool.as_str() {
            "nodejs" | "npm" => {
                let mut steps = Vec::new();
                if is_node_managed_by_nvm() {
                    log::info!("[uninstall] 检测到 nvm 管理的 node，使用 nvm uninstall");
                    steps.push(("__cmd__", "test -s ~/.nvm/nvm.sh && . ~/.nvm/nvm.sh 2>/dev/null && nvm uninstall $(nvm current) 2>&1 || true"));
                } else if is_node_managed_by_brew() {
                    log::info!("[uninstall] 检测到 brew 管理的 node，使用 brew uninstall");
                    steps.push(("__cmd__", "brew uninstall node 2>&1"));
                    steps.push(("__cmd__", "brew uninstall node@22 2>&1; brew uninstall node@20 2>&1; brew uninstall node@18 2>&1"));
                } else {
                    // 都不是，尝试全部
                    steps.push(("__cmd__", "brew uninstall node 2>&1"));
                    steps.push(("__cmd__", "test -s ~/.nvm/nvm.sh && . ~/.nvm/nvm.sh 2>/dev/null && nvm uninstall $(nvm current) 2>&1 || true"));
                }
                steps.push(("__rm__",  "/usr/local/bin/node"));
                steps.push(("__rm__",  "/opt/homebrew/bin/node"));
                steps.push(("__rm__",  "/usr/local/bin/npm"));
                steps.push(("__rm__",  "/opt/homebrew/bin/npm"));
                steps.push(("__rm_h__", "~/.npm"));
                steps.push(("__rm_h__", "~/.npm-cache"));
                steps
            },
            "python" => vec![
                ("__cmd__", "brew uninstall python 2>&1"),
                ("__cmd__", "brew uninstall python3 2>&1"),
                ("__rm__",  "/usr/local/bin/python3"),
                ("__rm__",  "/opt/homebrew/bin/python3"),
                ("__rm__",  "/usr/local/bin/pip3"),
                ("__rm__",  "/opt/homebrew/bin/pip3"),
            ],
            "git" => vec![
                ("__cmd__", "brew uninstall git 2>&1"),
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
            "claude" => {
                let mut steps = Vec::new();
                // 只有 npm 可用时才执行 npm uninstall
                if which::which("npm").is_ok() {
                    steps.push(("__cmd__", "npm uninstall -g @anthropic-ai/claude-code 2>&1"));
                }
                // 只删除 claude 可执行文件，不删除配置目录
                steps.push(("__rm_f__", "~/.local/bin/claude"));
                steps
            },
            _ => vec![],
        };

        let total = uninstall_steps.len() as u32;
        for (i, (kind, val)) in uninstall_steps.iter().enumerate() {
            let pct = ((i as u32 + 1) * 30) / total.max(1);
            emit(pct);
            match *kind {
                "__cmd__" => {
                    let _ = run_cmd_logged(&tool, "卸载", "/bin/zsh", &["-c", val], &path);
                }
                "__rm__" => {
                    let exists = std::path::Path::new(val).exists();
                    let ok = remove_dir_silent(val);
                    append_install_log(&tool, ok, &format!("[卸载] rm '{}' (存在: {})", val, exists));
                }
                "__rm_h__" => {
                    let expanded = val.replace("~", &home);
                    let exists = std::path::Path::new(&expanded).exists();
                    let ok = remove_dir_silent(&expanded);
                    append_install_log(&tool, ok, &format!("[卸载] rm '{}' (存在: {})", expanded, exists));
                }
                "__rm_f__" => {
                    let expanded = val.replace("~", &home);
                    let path = std::path::Path::new(&expanded);
                    let exists = path.exists();
                    let ok = if exists { std::fs::remove_file(path).is_ok() } else { true };
                    append_install_log(&tool, ok, &format!("[卸载] rm file '{}' (存在: {})", expanded, exists));
                }
                _ => {}
            }
        }

        std::thread::sleep(Duration::from_secs(2));
        } // end if uninstall_first

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

        let (result, _, _) = run_cmd_logged(&tool, "安装", cmd, &args, &path);
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

// ── 日志模块 ──────────────────────────────────────────────────────

use std::sync::OnceLock;

static LOGS_DIR: OnceLock<Option<std::path::PathBuf>> = OnceLock::new();

const LOG_RETENTION_DAYS: i64 = 7;

/// 获取日志目录路径（app_data_dir/logs/）
fn get_logs_dir() -> Option<&'static std::path::PathBuf> {
    LOGS_DIR
        .get_or_init(|| {
            let dir = dirs::data_local_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("DevClaw")
                .join("logs");
            if let Err(e) = std::fs::create_dir_all(&dir) {
                eprintln!("[env_log] 创建日志目录失败: {}", e);
                return None;
            }
            Some(dir)
        })
        .as_ref()
}

/// 初始化日志目录并清理过期日志（应在应用启动时调用）
pub fn init_log_dir() {
    let _ = get_logs_dir();
    cleanup_old_logs();
}

/// 清理超过 LOG_RETENTION_DAYS 天的旧日志文件
fn cleanup_old_logs() {
    if let Some(dir) = get_logs_dir() {
        let now = chrono::Local::now();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "log").unwrap_or(false) {
                    let mut should_delete = false;
                    // 按文件名日期清理：install-2026-05-22.log
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Some(date_str) = stem.strip_prefix("install-") {
                            if let Ok(file_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                                if let Some(file_dt) = file_date.and_hms_opt(23, 59, 59) {
                                    use chrono::TimeZone;
                                    if let Some(file_local) = chrono::Local.from_local_datetime(&file_dt).single() {
                                        if now.signed_duration_since(file_local).num_days() > LOG_RETENTION_DAYS {
                                            should_delete = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // 也按文件修改时间清理（兜底）
                    if !should_delete {
                        if let Ok(meta) = std::fs::metadata(&path) {
                            if let Ok(modified) = meta.modified() {
                                let age = std::time::SystemTime::now().duration_since(modified).unwrap_or_default();
                                if age.as_secs() > (LOG_RETENTION_DAYS * 86400) as u64 {
                                    should_delete = true;
                                }
                            }
                        }
                    }
                    if should_delete {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }
}

/// 获取当天安装日志路径：install-YYYY-MM-DD.log
fn get_install_log_path() -> std::path::PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("DevClaw")
        .join("logs");
    std::fs::create_dir_all(&dir).ok();
    let date = chrono::Local::now().format("%Y-%m-%d");
    dir.join(format!("install-{}.log", date))
}

/// 写入安装日志（带时间戳、工具名、详细信息）
pub fn write_install_log(tool: &str, success: bool, msg: &str) {
    use std::io::Write;
    let path = get_install_log_path();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let status = if success { "成功" } else { "失败" };
    let line = format!("[{}] {} - {} {}\n", now, tool, status, msg);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}

/// 打开日志目录
#[tauri::command]
pub async fn open_logs_dir(handle: tauri::AppHandle) -> Result<bool, String> {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("DevClaw")
        .join("logs");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    handle
        .opener()
        .open_path(dir.to_string_lossy().to_string(), None::<String>)
        .map_err(|e| format!("打开文件夹失败: {e}"))?;
    Ok(true)
}

/// 获取安装日志内容（只读 install-*.log，自动清理过期日志）
#[tauri::command]
pub async fn get_logs_content() -> Result<String, String> {
    // 先清理过期日志
    cleanup_old_logs();

    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("DevClaw")
        .join("logs");
    if !dir.exists() {
        return Ok("暂无安装日志".to_string());
    }
    let mut output = String::new();
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| format!("读取目录失败: {e}"))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with("install-") && name.ends_with(".log")
        })
        .collect();
    files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    for entry in files {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        output.push_str(&format!("=== {} ===\n", name));
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let start = if lines.len() > 500 {
                    lines.len() - 500
                } else {
                    0
                };
                if start > 0 {
                    output.push_str(&format!("... (省略 {} 行)\n", start));
                }
                for line in &lines[start..] {
                    output.push_str(line);
                    output.push('\n');
                }
            }
            Err(e) => {
                output.push_str(&format!("读取失败: {}\n", e));
            }
        }
        output.push('\n');
    }
    if output.is_empty() {
        Ok("暂无安装日志".to_string())
    } else {
        Ok(output)
    }
}

/// 调试命令
#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn debug_env() -> String {
    tauri::async_runtime::spawn_blocking(|| debug_env_inner()).await.unwrap_or_default()
}

#[cfg(target_os = "windows")]
fn debug_env_inner() -> String {
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

/// 调试命令 (macOS / Linux)
#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub async fn debug_env() -> String {
    tauri::async_runtime::spawn_blocking(|| debug_env_inner()).await.unwrap_or_default()
}

#[cfg(not(target_os = "windows"))]
fn debug_env_inner() -> String {
    let mut out = String::new();
    let mut log_line = |label: &str, msg: &str| {
        let line = format!("[{}] {}", label, msg);
        eprintln!("{}", line);
        out.push_str(&line);
        out.push('\n');
    };

    log_line("=== debug_env ===", "");

    // 进程 PATH
    let proc_path = std::env::var("PATH").unwrap_or_else(|e| format!("<读取失败: {}>", e));
    log_line("进程PATH", &proc_path);

    // SHELL 环境变量
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "<未设置>".to_string());
    log_line("SHELL", &shell);

    // login shell PATH
    let shell_path = get_shell_path();
    log_line("合并后PATH", &shell_path);
    let path_count = shell_path.split(':').filter(|p| !p.is_empty()).count();
    log_line("PATH目录数", &path_count.to_string());

    // which 检测
    log_line("--- which 检测 ---", "");
    for tool in &["node", "npm", "git", "python3", "code", "claude"] {
        match which::which(tool) {
            Ok(p) => log_line(&format!("which({})", tool), &p.display().to_string()),
            Err(e) => log_line(&format!("which({})", tool), &format!("未找到: {}", e)),
        }
    }

    // 直接命令执行
    log_line("--- 命令执行 ---", "");
    for (name, cmd, args) in [
        ("node", "node", vec!["--version"]),
        ("npm", "npm", vec!["--version"]),
        ("git", "git", vec!["--version"]),
        ("python3", "python3", vec!["--version"]),
        ("claude", "claude", vec!["--version"]),
    ] {
        match Command::new(cmd).args(&args).env("PATH", &shell_path).output() {
            Ok(o) => log_line(
                &format!("{} --version", name),
                &format!(
                    "exit={:?} stdout=[{}] stderr=[{}]",
                    o.status.code(),
                    String::from_utf8_lossy(&o.stdout).trim(),
                    String::from_utf8_lossy(&o.stderr).trim()
                ),
            ),
            Err(e) => log_line(&format!("{} --version", name), &format!("错误: {}", e)),
        }
    }

    // 关键 shell 配置文件
    log_line("--- shell 配置文件 ---", "");
    let home = std::env::var("HOME").unwrap_or_default();
    for f in &[
        format!("{}/.zshrc", home),
        format!("{}/.zprofile", home),
        format!("{}/.bashrc", home),
        format!("{}/.bash_profile", home),
        format!("{}/.profile", home),
    ] {
        if std::path::Path::new(f).exists() {
            let path_lines = std::fs::read_to_string(f)
                .map(|c| {
                    c.lines()
                        .filter(|l| l.contains("PATH"))
                        .take(5)
                        .collect::<Vec<_>>()
                        .join(" | ")
                })
                .unwrap_or_else(|e| format!("读取失败: {}", e));
            log_line(&format!("存在: {}", f), &path_lines);
        } else {
            log_line(f, "不存在");
        }
    }

    log_line("=== debug_env 结束 ===", "");
    out
}
