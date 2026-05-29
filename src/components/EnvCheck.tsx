import { useEffect, useState, useCallback } from "react";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import {
  Loader2,
  CheckCircle2,
  AlertCircle,
  ExternalLink,
  ArrowRight,
  Monitor,
  RefreshCw,
  Download,
  ChevronDown,
  Stethoscope,
} from "lucide-react";
import { Button } from "@/components/ui/button";

interface EnvCheckProps {
  onDone: () => void;
}

interface EnvItem {
  key: string;
  name: string;
  iconUrl: string;
  fallbackLetter: string;
  fallbackColor: string;
  downloadUrl: string;
}

const ALL_TOOLS: EnvItem[] = [
  { key: "nodejs", name: "Node.js", iconUrl: "https://cdn.simpleicons.org/nodedotjs", fallbackLetter: "N", fallbackColor: "#339933", downloadUrl: "https://nodejs.org" },
  { key: "npm", name: "npm", iconUrl: "https://cdn.simpleicons.org/npm", fallbackLetter: "n", fallbackColor: "#CB3837", downloadUrl: "https://nodejs.org" },
  { key: "git", name: "Git", iconUrl: "https://cdn.simpleicons.org/git", fallbackLetter: "G", fallbackColor: "#F05032", downloadUrl: "https://git-scm.com" },
  { key: "python", name: "Python", iconUrl: "https://cdn.simpleicons.org/python", fallbackLetter: "P", fallbackColor: "#3776AB", downloadUrl: "https://python.org" },
  { key: "vscode", name: "VS Code", iconUrl: "https://cdn.simpleicons.org/visualstudiocode", fallbackLetter: "V", fallbackColor: "#007ACC", downloadUrl: "https://code.visualstudio.com" },
  { key: "chrome", name: "Chrome", iconUrl: "https://cdn.simpleicons.org/googlechrome", fallbackLetter: "C", fallbackColor: "#4285F4", downloadUrl: "https://chrome.google.com" },
  { key: "claude", name: "Claude CLI", iconUrl: "", fallbackLetter: "C", fallbackColor: "#D97757", downloadUrl: "https://claude.ai/download" },
];

interface NpmAvailability {
  available: boolean;
  checked: boolean;
}

function ToolIcon({ item }: { item: EnvItem }) {
  const [failed, setFailed] = useState(!item.iconUrl);
  return failed ? (
    <div
      className="w-8 h-8 shrink-0 rounded-full flex items-center justify-center text-white text-sm font-bold"
      style={{ backgroundColor: item.fallbackColor }}
    >
      {item.fallbackLetter}
    </div>
  ) : (
    <img
      src={item.iconUrl}
      alt={item.name}
      className="w-8 h-8 shrink-0 object-contain"
      onError={() => setFailed(true)}
    />
  );
}

export function EnvCheck({ onDone }: EnvCheckProps) {
  const [versions, setVersions] = useState<Record<string, string | null>>({});
  const [loading, setLoading] = useState(true);
  const [npmAvail, setNpmAvail] = useState<NpmAvailability>({ available: false, checked: false });
  const [installing, setInstalling] = useState<Record<string, boolean>>({});
  const [installMsg, setInstallMsg] = useState<Record<string, string | null>>({});
  const [debugOutput, setDebugOutput] = useState<string | null>(null);
  const [debugLoading, setDebugLoading] = useState(false);
  const [showDebug, setShowDebug] = useState(false);

  const fetchAll = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<Record<string, string | null>>("check_env");
      setVersions(result);
      if (result.npm) {
        try {
          const ok = await invoke<boolean>("check_npm_available");
          setNpmAvail({ available: ok, checked: true });
        } catch {
          setNpmAvail({ available: false, checked: true });
        }
      } else {
        setNpmAvail({ available: false, checked: true });
      }
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchAll();
  }, [fetchAll]);

  const openUrl = async (url: string) => {
    try {
      await invoke("open_external", { url });
    } catch {
      window.open(url, "_blank");
    }
  };

  const handleInstall = async (toolKey: string) => {
    setInstalling((s) => ({ ...s, [toolKey]: true }));
    setInstallMsg((s) => ({ ...s, [toolKey]: null }));
    try {
      const [success, msg] = await invoke<[boolean, string]>("install_tool", { tool: toolKey });
      setInstallMsg((s) => ({ ...s, [toolKey]: msg }));
      if (success) {
        const newVer = await invoke<string | null>("check_single_env", { tool: toolKey });
        setVersions((s) => ({ ...s, [toolKey]: newVer }));
        if (toolKey === "npm" || toolKey === "nodejs") {
          try {
            const ok = await invoke<boolean>("check_npm_available");
            setNpmAvail({ available: ok, checked: true });
          } catch {
            setNpmAvail({ available: false, checked: true });
          }
        }
      }
    } catch (e: any) {
      setInstallMsg((s) => ({ ...s, [toolKey]: `错误: ${e?.toString() || "未知错误"}` }));
    } finally {
      setInstalling((s) => ({ ...s, [toolKey]: false }));
    }
  };

  const handleDebug = async () => {
    setDebugLoading(true);
    try {
      const result = await invoke<string>("debug_env");
      setDebugOutput(result);
      setShowDebug(true);
    } catch (e: any) {
      setDebugOutput(`调用失败: ${e?.toString()}`);
      setShowDebug(true);
    } finally {
      setDebugLoading(false);
    }
  };

  const installedCount = Object.values(versions).filter(Boolean).length;
  const totalCount = ALL_TOOLS.length;

  return (
    <div className="flex items-center justify-center min-h-screen w-screen bg-background px-6 py-8">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4, ease: "easeOut" }}
        className="w-full max-w-3xl flex flex-col gap-6"
      >
        {/* Header */}
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3, delay: 0.05 }}
          className="rounded-xl border border-border bg-gradient-to-br from-card/80 to-card/40 p-6 shadow-sm"
        >
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 rounded-xl bg-blue-600 flex items-center justify-center shrink-0">
              <Monitor className="w-6 h-6 text-white" />
            </div>
            <div className="flex-1 min-w-0">
              <h1 className="text-lg font-semibold text-foreground">环境检测</h1>
              <p className="text-sm text-muted-foreground flex items-center gap-1.5 flex-wrap">
                检测开发环境依赖的安装状态
                {!loading && (
                  <span className="text-muted-foreground">
                    （{installedCount}/{totalCount} 已安装）
                  </span>
                )}
              </p>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => fetchAll()}
              disabled={loading}
              className="h-8 gap-1.5 text-xs shrink-0"
            >
              <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
              重新检测
            </Button>
          </div>

          {!loading && (
            <div className="flex items-center gap-2 mt-4">
              {installedCount === totalCount ? (
                <span className="text-xs text-green-500 flex items-center gap-1">
                  <CheckCircle2 className="w-3.5 h-3.5" />
                  所有工具已安装
                </span>
              ) : (
                <span className="text-xs text-muted-foreground flex items-center gap-1">
                  <AlertCircle className="w-3.5 h-3.5 text-yellow-500" />
                  部分工具未安装，可点击"安装"或"官网"按钮
                </span>
              )}
            </div>
          )}
        </motion.div>

        {/* Tool cards grid */}
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {ALL_TOOLS.map((item, index) => {
            const version = loading ? undefined : versions[item.key];
            const installed = version !== undefined && version !== null;
            const isInstalling = installing[item.key];
            const msg = installMsg[item.key];

            return (
              <motion.div
                key={item.key}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.3, delay: 0.1 + index * 0.04 }}
                className="flex min-h-[130px] flex-col gap-3 rounded-xl border border-border bg-gradient-to-br from-card/80 to-card/40 p-4 shadow-sm transition-colors hover:border-primary/30"
              >
                {/* Top row: icon + name + status */}
                <div className="flex items-start justify-between gap-3">
                  <div className="flex items-center gap-2.5 min-w-0">
                    <ToolIcon item={item} />
                    <span className="text-sm font-medium text-foreground truncate">
                      {item.name}
                    </span>
                  </div>
                  {loading ? (
                    <Loader2 className="mt-0.5 h-4 w-4 animate-spin text-muted-foreground shrink-0" />
                  ) : installed ? (
                    <CheckCircle2 className="mt-0.5 h-4 w-4 text-green-500 shrink-0" />
                  ) : (
                    <AlertCircle className="mt-0.5 h-4 w-4 text-yellow-500 shrink-0" />
                  )}
                </div>

                {/* Version info */}
                <div className="space-y-1 text-xs">
                  {loading ? (
                    <span className="text-muted-foreground">检测中...</span>
                  ) : installed ? (
                    <div className="flex items-center gap-1.5">
                      <span className="font-mono text-green-600 dark:text-green-400">
                        {version}
                      </span>
                      {item.key === "npm" && npmAvail.checked && (
                        <span
                          className={`text-[10px] px-1 py-0.5 rounded-full border ${
                            npmAvail.available
                              ? "text-green-600 dark:text-green-400 border-green-500/20 bg-green-500/10"
                              : "text-red-500 border-red-500/20 bg-red-500/10"
                          }`}
                        >
                          {npmAvail.available ? "可用" : "不可用"}
                        </span>
                      )}
                    </div>
                  ) : (
                    <span className="text-muted-foreground">未检测到</span>
                  )}
                </div>

                {/* Action buttons */}
                <div className="mt-auto flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-7 gap-1 text-xs flex-1"
                    disabled={!!isInstalling}
                    onClick={() => handleInstall(item.key)}
                  >
                    {isInstalling ? (
                      <Loader2 className="h-3 w-3 animate-spin" />
                    ) : (
                      <Download className="h-3 w-3" />
                    )}
                    {isInstalling ? "安装中..." : installed ? "重装" : "安装"}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-7 gap-1 text-xs flex-1"
                    onClick={() => openUrl(item.downloadUrl)}
                  >
                    <ExternalLink className="h-3 w-3" />
                    官网
                  </Button>
                </div>

                {/* Install message */}
                {msg && (
                  <div
                    className={`text-[11px] px-2 py-1 rounded ${
                      msg.includes("成功")
                        ? "text-green-600 dark:text-green-400 bg-green-500/10"
                        : "text-red-500 bg-red-500/10"
                    }`}
                  >
                    {msg.split("\n")[0]}
                  </div>
                )}
              </motion.div>
            );
          })}
        </div>

        {/* Debug section */}
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3, delay: 0.4 }}
          className="rounded-xl border border-border bg-gradient-to-br from-card/80 to-card/40 p-4 shadow-sm"
        >
          <button
            type="button"
            onClick={() => {
              if (!showDebug && !debugOutput) {
                handleDebug();
              } else {
                setShowDebug((v) => !v);
              }
            }}
            className="flex w-full items-center gap-2 text-sm font-medium text-foreground transition-colors hover:text-primary"
          >
            <ChevronDown
              className={`h-4 w-4 transition-transform ${showDebug ? "" : "-rotate-90"}`}
            />
            <Stethoscope className="h-4 w-4" />
            安装环境诊断
            {debugLoading && <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground" />}
          </button>

          {showDebug && debugOutput && (
            <div className="mt-3 rounded-lg border border-border/60 bg-background/80 p-4">
              <pre className="text-xs font-mono text-foreground whitespace-pre-wrap break-all leading-relaxed max-h-96 overflow-auto">
                {debugOutput}
              </pre>
            </div>
          )}
        </motion.div>

        {/* Enter button */}
        <Button onClick={onDone} size="lg" className="w-full">
          完成，进入 DevClaw
          <ArrowRight className="w-4 h-4 ml-2" />
        </Button>
      </motion.div>
    </div>
  );
}
