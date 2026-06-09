import { useState, useCallback } from "react";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import {
  Settings,
  TestTube2,
  Info,
  FolderOpen,
  Loader2,
  CheckCircle2,
  XCircle,
  Trash2,
  Shield,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

interface Preset {
  label: string;
  httpPort: string;
  socks5Port: string;
}

const PRESETS: Preset[] = [
  { label: "Clash Verge", httpPort: "7897", socks5Port: "7897" },
  { label: "Clash for Windows", httpPort: "7890", socks5Port: "7890" },
  { label: "Clash Meta", httpPort: "7890", socks5Port: "7890" },
  { label: "V2RayN", httpPort: "10809", socks5Port: "10808" },
  { label: "Shadowsocks", httpPort: "1080", socks5Port: "1080" },
  { label: "Surge", httpPort: "6152", socks5Port: "6152" },
  { label: "Nekoray", httpPort: "1080", socks5Port: "1080" },
  { label: "Proxifier", httpPort: "1080", socks5Port: "1080" },
];

export function ClaudeProxy() {
  const [httpPort, setHttpPort] = useState("");
  const [socks5Port, setSocks5Port] = useState("");
  const [currentConfig, setCurrentConfig] = useState<{
    http_port: string;
    socks5_port: string;
  } | null>(null);

  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<"success" | "fail" | null>(null);
  const [testMessage, setTestMessage] = useState("");
  const [ipInfo, setIpInfo] = useState<string | null>(null);

  const [saving, setSaving] = useState(false);
  const [clearing, setClearing] = useState(false);
  const [loadingConfig, setLoadingConfig] = useState(true);

  const loadConfig = useCallback(async () => {
    setLoadingConfig(true);
    try {
      const info = await invoke<{
        http_port: string;
        socks5_port: string;
      } | null>("get_claude_proxy");
      if (info) {
        setCurrentConfig(info);
        setHttpPort(info.http_port);
        setSocks5Port(info.socks5_port);
      } else {
        setCurrentConfig(null);
      }
    } catch {
      // ignore
    } finally {
      setLoadingConfig(false);
    }
  }, []);

  useState(() => {
    loadConfig();
  });

  const handleSave = async () => {
    if (!httpPort && !socks5Port) {
      toast.error("请至少填写一个端口");
      return;
    }
    setSaving(true);
    try {
      await invoke("set_claude_proxy", { httpPort, socks5Port });
      toast.success("代理配置已保存");
      await loadConfig();
    } catch (e) {
      toast.error(`保存失败: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const handleClear = async () => {
    setClearing(true);
    try {
      await invoke("clear_claude_proxy");
      toast.success("代理配置已清除");
      setHttpPort("");
      setSocks5Port("");
      setCurrentConfig(null);
    } catch (e) {
      toast.error(`清除失败: ${e}`);
    } finally {
      setClearing(false);
    }
  };

  const handleTest = async () => {
    if (!httpPort && !socks5Port) {
      toast.error("请至少填写一个端口");
      return;
    }
    setTesting(true);
    setTestResult(null);
    setTestMessage("");
    setIpInfo(null);
    try {
      // Test HTTP first, then SOCKS5
      const results: string[] = [];
      if (httpPort) {
        const msg = await invoke<string>("test_proxy", {
          proxyType: "http",
          host: "127.0.0.1",
          port: httpPort,
        });
        results.push(`HTTP: ${msg}`);
      }
      if (socks5Port) {
        const msg = await invoke<string>("test_proxy", {
          proxyType: "socks5",
          host: "127.0.0.1",
          port: socks5Port,
        });
        results.push(`SOCKS5: ${msg}`);
      }
      setTestResult("success");
      setTestMessage(results.join("\n"));

      // Auto-fetch IP after test
      try {
        const info = await invoke<string>("get_current_ip", {
          proxyType: httpPort ? "http" : "socks5",
          host: "127.0.0.1",
          port: httpPort || socks5Port,
        });
        setIpInfo(info);
      } catch {
        // IP fetch failure is non-critical
      }
    } catch (e) {
      setTestResult("fail");
      setTestMessage(String(e));
    } finally {
      setTesting(false);
    }
  };

  const handleOpenDir = async () => {
    try {
      await invoke("open_claude_dir");
    } catch (e) {
      toast.error(`打开失败: ${e}`);
    }
  };

  const handlePreset = (preset: Preset) => {
    setHttpPort(preset.httpPort);
    setSocks5Port(preset.socks5Port);
  };

  const hasConfig =
    currentConfig && (currentConfig.http_port || currentConfig.socks5_port);

  return (
    <div className="relative w-full h-full flex flex-col bg-background">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4, ease: "easeOut" }}
        className="flex-1 overflow-y-auto scroll-overlay"
      >
        <div className="px-6 py-6 space-y-6 w-full max-w-3xl mx-auto">
          {/* Block 1: Proxy Settings */}
          <div className="rounded-lg border bg-card shadow-sm">
            <div className="p-6 space-y-4">
              <div className="flex items-center gap-2 text-sm font-medium text-foreground">
                <Settings className="w-4 h-4" />
                代理设置
              </div>

              {/* Two-column port inputs */}
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1.5">
                  <span className="text-xs text-muted-foreground">
                    HTTP (HTTPS)
                  </span>
                  <Input
                    placeholder="端口，如 7890"
                    value={httpPort}
                    onChange={(e) => setHttpPort(e.target.value)}
                    className="h-9"
                  />
                </div>
                <div className="space-y-1.5">
                  <span className="text-xs text-muted-foreground">SOCKS5</span>
                  <Input
                    placeholder="端口，如 7890"
                    value={socks5Port}
                    onChange={(e) => setSocks5Port(e.target.value)}
                    className="h-9"
                  />
                </div>
              </div>

              {/* Preset quick select */}
              <div className="space-y-1.5">
                <span className="text-xs text-muted-foreground">
                  快速选择（点击填入端口）
                </span>
                <div className="flex flex-wrap gap-1.5">
                  {PRESETS.map((p) => (
                    <Button
                      key={p.label}
                      variant="outline"
                      size="sm"
                      className="h-7 text-xs px-2"
                      onClick={() => handlePreset(p)}
                    >
                      {p.label} {p.httpPort}/{p.socks5Port}
                    </Button>
                  ))}
                </div>
              </div>

              {/* Action buttons */}
              <div className="flex gap-2 pt-1">
                <Button
                  size="sm"
                  className="h-8 text-xs"
                  onClick={handleSave}
                  disabled={saving}
                >
                  {saving ? (
                    <Loader2 className="w-3.5 h-3.5 animate-spin mr-1" />
                  ) : (
                    <CheckCircle2 className="w-3.5 h-3.5 mr-1" />
                  )}
                  保存
                </Button>
                <Button
                  variant="destructive"
                  size="sm"
                  className="h-8 text-xs"
                  onClick={handleClear}
                  disabled={clearing}
                >
                  {clearing ? (
                    <Loader2 className="w-3.5 h-3.5 animate-spin mr-1" />
                  ) : (
                    <Trash2 className="w-3.5 h-3.5 mr-1" />
                  )}
                  清除代理
                </Button>
              </div>
            </div>
          </div>

          {/* Block 2: Connection Test */}
          <div className="rounded-lg border bg-card shadow-sm">
            <div className="p-6 space-y-4">
              <div className="flex items-center gap-2 text-sm font-medium text-foreground">
                <TestTube2 className="w-4 h-4" />
                连接测试
              </div>

              <Button
                variant="outline"
                size="sm"
                className="h-8 text-xs"
                onClick={handleTest}
                disabled={testing || (!httpPort && !socks5Port)}
              >
                {testing ? (
                  <Loader2 className="w-3.5 h-3.5 animate-spin mr-1" />
                ) : (
                  <TestTube2 className="w-3.5 h-3.5 mr-1" />
                )}
                测试连接
              </Button>

              {testResult && (
                <div
                  className={`flex items-start gap-2 p-3 rounded-md text-xs ${
                    testResult === "success"
                      ? "bg-green-500/10 border border-green-500/30 text-green-500"
                      : "bg-red-500/10 border border-red-500/30 text-red-500"
                  }`}
                >
                  {testResult === "success" ? (
                    <CheckCircle2 className="w-4 h-4 shrink-0 mt-0.5" />
                  ) : (
                    <XCircle className="w-4 h-4 shrink-0 mt-0.5" />
                  )}
                  <span>{testMessage}</span>
                </div>
              )}

              {ipInfo && (
                <div className="p-3 rounded-md bg-muted text-xs font-mono whitespace-pre-wrap">
                  {ipInfo}
                </div>
              )}
            </div>
          </div>

          {/* Block 3: Current Config */}
          <div className="rounded-lg border bg-card shadow-sm">
            <div className="p-6 space-y-3">
              <div className="flex items-center gap-2 text-sm font-medium text-foreground">
                <Info className="w-4 h-4" />
                当前配置状态
              </div>

              {loadingConfig ? (
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  加载中...
                </div>
              ) : hasConfig ? (
                <div className="space-y-2">
                  <div className="flex items-center gap-2 text-xs">
                    <Shield className="w-3.5 h-3.5 text-green-500" />
                    <span className="text-green-500 font-medium">
                      已配置代理
                    </span>
                  </div>
                  <div className="grid grid-cols-2 gap-2 text-xs">
                    {currentConfig.http_port && (
                      <div className="p-2 rounded bg-muted">
                        <span className="text-muted-foreground">
                          HTTP (HTTPS)
                        </span>
                        <div className="font-mono font-medium mt-0.5">
                          127.0.0.1:{currentConfig.http_port}
                        </div>
                      </div>
                    )}
                    {currentConfig.socks5_port && (
                      <div className="p-2 rounded bg-muted">
                        <span className="text-muted-foreground">SOCKS5</span>
                        <div className="font-mono font-medium mt-0.5">
                          127.0.0.1:{currentConfig.socks5_port}
                        </div>
                      </div>
                    )}
                  </div>
                  <p className="text-[11px] text-muted-foreground">
                    写入 ~/.claude/settings.json 的 env 字段
                  </p>
                </div>
              ) : (
                <div className="space-y-2">
                  <div className="flex items-center gap-2 text-xs">
                    <XCircle className="w-3.5 h-3.5 text-muted-foreground" />
                    <span className="text-muted-foreground">未配置代理</span>
                  </div>
                  <p className="text-[11px] text-muted-foreground">
                    Claude CLI 当前使用默认网络连接
                  </p>
                </div>
              )}
            </div>
          </div>

          {/* Block 4: Open .claude directory */}
          <div className="rounded-lg border bg-card shadow-sm">
            <div className="p-6 space-y-3">
              <div className="flex items-center gap-2 text-sm font-medium text-foreground">
                <FolderOpen className="w-4 h-4" />
                Claude 配置目录
              </div>
              <p className="text-xs text-muted-foreground">
                打开 ~/.claude 目录，可直接编辑 settings.json
              </p>
              <Button
                variant="outline"
                size="sm"
                className="h-8 text-xs"
                onClick={handleOpenDir}
              >
                <FolderOpen className="w-3.5 h-3.5 mr-1" />
                打开 .claude 目录
              </Button>
            </div>
          </div>
        </div>
      </motion.div>
    </div>
  );
}
