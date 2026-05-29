import { useEffect, useState, useCallback } from "react";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Loader2,
  CheckCircle2,
  AlertCircle,
  ExternalLink,
  ArrowRight,
  Monitor,
  Terminal,
  RefreshCw,
  Wrench,
  Download,
  ChevronDown,
  Stethoscope,
  SquareCheck,
  Square,
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

const BASIC_ENV: EnvItem[] = [
  { key: "nodejs", name: "Node.js", iconUrl: "https://cdn.simpleicons.org/nodedotjs", fallbackLetter: "N", fallbackColor: "#339933", downloadUrl: "https://nodejs.org" },
  { key: "npm", name: "npm", iconUrl: "https://cdn.simpleicons.org/npm", fallbackLetter: "n", fallbackColor: "#CB3837", downloadUrl: "https://nodejs.org" },
  { key: "git", name: "Git", iconUrl: "https://cdn.simpleicons.org/git", fallbackLetter: "G", fallbackColor: "#F05032", downloadUrl: "https://git-scm.com" },
  { key: "python", name: "Python", iconUrl: "https://cdn.simpleicons.org/python", fallbackLetter: "P", fallbackColor: "#3776AB", downloadUrl: "https://python.org" },
];

const TOOLS: EnvItem[] = [
  { key: "vscode", name: "VS Code", iconUrl: "https://cdn.simpleicons.org/visualstudiocode", fallbackLetter: "V", fallbackColor: "#007ACC", downloadUrl: "https://code.visualstudio.com" },
  { key: "chrome", name: "Chrome", iconUrl: "https://cdn.simpleicons.org/googlechrome", fallbackLetter: "C", fallbackColor: "#4285F4", downloadUrl: "https://chrome.google.com" },
  { key: "claude", name: "Claude CLI", iconUrl: "", fallbackLetter: "C", fallbackColor: "#D97757", downloadUrl: "https://claude.ai/download" },
];

const ALL_TOOLS = [...BASIC_ENV, ...TOOLS];

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
  const [installProgress, setInstallProgress] = useState<Record<string, number>>({});
  const [fixInProgress, setFixInProgress] = useState(false);
  const [selectedTools, setSelectedTools] = useState<Record<string, boolean>>({});
  const [batchInstalling, setBatchInstalling] = useState(false);
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
    if (loading) return;
    const init: Record<string, boolean> = {};
    ALL_TOOLS.forEach((item) => {
      if (item.key === "npm") return;
      const v = versions[item.key];
      init[item.key] = v === undefined || v === null;
    });
    setSelectedTools(init);
  }, [loading, versions]);

  useEffect(() => {
    fetchAll();
  }, [fetchAll]);

  useEffect(() => {
    const unlisten = listen<{ tool: string; progress: number; done: boolean }>(
      "install_progress",
      (event) => {
        const { tool, progress, done } = event.payload;
        setInstallProgress((s) => ({ ...s, [tool]: progress }));
        if (done) {
          setTimeout(async () => {
            await fetchAll();
            setInstalling((s) => ({ ...s, [tool]: false }));
            setInstallProgress((s) => {
              const next = { ...s };
              delete next[tool];
              return next;
            });
          }, 1000);
        }
      }
    );
    return () => {
      unlisten.then((fn) => fn());
    };
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
    setInstallProgress((s) => ({ ...s, [toolKey]: 0 }));
    const version = versions[toolKey];
    const uninstallFirst = version !== undefined && version !== null;
    try {
      await invoke("install_tool", { tool: toolKey, uninstallFirst });
    } catch (e: any) {
      console.error("install error:", e);
      setInstalling((s) => ({ ...s, [toolKey]: false }));
      setInstallProgress((s) => {
        const next = { ...s };
        delete next[toolKey];
        return next;
      });
    }
  };

  const handleFixNpm = async () => {
    setFixInProgress(true);
    try {
      const [success] = await invoke<[boolean, string]>("fix_npm_registry");
      if (success) {
        const ok = await invoke<boolean>("check_npm_available");
        setNpmAvail({ available: ok, checked: true });
      }
    } catch {
      // ignore
    } finally {
      setFixInProgress(false);
    }
  };

  const handleBatchInstall = async () => {
    const tools = Object.entries(selectedTools)
      .filter(([, v]) => v)
      .map(([k]) => k);
    if (tools.length === 0) return;
    setBatchInstalling(true);
    for (const toolKey of tools) {
      await handleInstall(toolKey);
      await new Promise<void>((resolve) => {
        const check = setInterval(() => {
          setInstalling((s) => {
            if (!s[toolKey]) {
              clearInterval(check);
              resolve();
            }
            return s;
          });
        }, 200);
      });
    }
    setBatchInstalling(false);
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

  const renderStatusText = (item: EnvItem) => {
    if (loading) {
      return (
        <span className="text-xs text-muted-foreground flex items-center gap-1">
          <Loader2 className="w-3 h-3 animate-spin" />
          检测中
        </span>
      );
    }
    const version = versions[item.key];
    const installed = version !== undefined && version !== null;
    if (!installed) {
      return <span className="text-xs text-muted-foreground">未检测到</span>;
    }
    return (
      <span className="text-xs text-green-500 font-mono truncate" title={version}>
        {version}
      </span>
    );
  };

  const renderToolRow = (item: EnvItem) => {
    const version = loading ? undefined : versions[item.key];
    const installed = version !== undefined && version !== null;
    const isInstalling = installing[item.key];
    const progress = installProgress[item.key];
    const isNpm = item.key === "npm";
    return (
      <div key={item.key} className="flex items-center gap-3 py-2">
        {item.key === "npm" ? (
          <div className="w-4 h-4 shrink-0" />
        ) : (
          <button
            onClick={() => setSelectedTools((s) => ({ ...s, [item.key]: !s[item.key] }))}
            className="w-4 h-4 shrink-0 flex items-center justify-center text-muted-foreground hover:text-foreground transition-colors"
          >
            {selectedTools[item.key] ? (
              <SquareCheck className="w-4 h-4 text-blue-500" />
            ) : (
              <Square className="w-4 h-4" />
            )}
          </button>
        )}
        <ToolIcon item={item} />
        <div className="flex flex-col min-w-0 flex-1 gap-0.5">
          <span className="text-sm font-medium text-foreground truncate">
            {item.name}
          </span>
          {renderStatusText(item)}
        </div>
        <div className="flex flex-row gap-1.5 shrink-0">
          {isNpm ? (
            <>
              {installed && npmAvail.checked ? (
                <span className={`h-7 flex items-center justify-center text-xs px-2 rounded-md border font-medium whitespace-nowrap ${
                  npmAvail.available
                    ? "bg-green-500/10 border-green-500/30 text-green-500"
                    : "bg-red-500/10 border-red-500/30 text-red-500"
                }`}>
                  {npmAvail.available ? "可用" : "不可用"}
                </span>
              ) : (
                <span className="h-7 flex items-center justify-center text-xs px-2 text-muted-foreground whitespace-nowrap">
                  检测中...
                </span>
              )}
              {installed && npmAvail.checked && !npmAvail.available && (
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 text-xs px-2 text-orange-500 border-orange-500/30 hover:bg-orange-500/10 whitespace-nowrap gap-1"
                  disabled={fixInProgress}
                  onClick={handleFixNpm}
                >
                  {fixInProgress ? (
                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  ) : (
                    <Wrench className="w-3.5 h-3.5" />
                  )}
                  修复
                </Button>
              )}
            </>
          ) : (
            <>
              <div className="flex items-center gap-1.5">
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 text-xs px-2 whitespace-nowrap gap-1"
                  disabled={!!isInstalling}
                  onClick={() => handleInstall(item.key)}
                >
                  {isInstalling ? (
                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  ) : (
                    <Terminal className="w-3.5 h-3.5" />
                  )}
                  {installed ? "重装" : "安装"}
                </Button>
                {isInstalling && progress !== undefined && (
                  <span className="text-[11px] text-muted-foreground whitespace-nowrap">
                    {progress}%
                  </span>
                )}
              </div>
              <Button
                variant="outline"
                size="sm"
                className="h-7 text-xs px-2 whitespace-nowrap gap-1"
                onClick={() => openUrl(item.downloadUrl)}
              >
                <ExternalLink className="w-3.5 h-3.5" />
                官网
              </Button>
            </>
          )}
        </div>
      </div>
    );
  };

  return (
    <div className="relative w-full h-full flex flex-col bg-background">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4, ease: "easeOut" }}
        className="flex-1 overflow-y-auto scroll-overlay"
      >
        <div className="px-6 py-6 space-y-6 w-full max-w-5xl mx-auto">
          {/* Header */}
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

          {/* Status bar */}
          {!loading && (
            <div className="flex items-center gap-2">
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

          {/* Two-column grid */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {/* Left: Basic environment */}
            <div className="rounded-xl border border-border bg-gradient-to-br from-card/80 to-card/40 p-4 shadow-sm">
              <h2 className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-2">
                基础环境
              </h2>
              <div className="divide-y divide-border/40">
                {BASIC_ENV.map(renderToolRow)}
              </div>
            </div>

            {/* Right: Tools */}
            <div className="rounded-xl border border-border bg-gradient-to-br from-card/80 to-card/40 p-4 shadow-sm">
              <h2 className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-2">
                工具
              </h2>
              <div className="divide-y divide-border/40">
                {TOOLS.map(renderToolRow)}
              </div>
            </div>
          </div>

          {/* Batch install */}
          <Button
            variant="default"
            size="sm"
            className="w-full h-9 text-sm"
            disabled={batchInstalling || loading || !Object.values(selectedTools).some(Boolean)}
            onClick={handleBatchInstall}
          >
            {batchInstalling ? (
              <Loader2 className="w-4 h-4 animate-spin mr-1.5" />
            ) : (
              <Download className="w-4 h-4 mr-1.5" />
            )}
            一键安装（{Object.values(selectedTools).filter(Boolean).length} 项）
          </Button>

          {/* Debug section */}
          <div className="rounded-xl border border-border bg-gradient-to-br from-card/80 to-card/40 p-4 shadow-sm">
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
          </div>

          {/* Enter button */}
          <Button onClick={onDone} size="lg" className="w-full">
            完成，进入 DevClaw
            <ArrowRight className="w-4 h-4 ml-2" />
          </Button>
        </div>
      </motion.div>
    </div>
  );
}
