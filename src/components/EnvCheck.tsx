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
  X,
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
  const [installProgress, setInstallProgress] = useState<Record<string, number>>({});
  const [fixInProgress, setFixInProgress] = useState(false);
  const [selectedTools, setSelectedTools] = useState<Record<string, boolean>>({});
  const [batchInstalling, setBatchInstalling] = useState(false);
  const [installModal, setInstallModal] = useState<{
    tool: string; logLines: string[]; progress: number; done: boolean;
  } | null>(null);
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

  // 检测完成后，默认勾选未安装的工具（npm 随 nodejs 自动安装，跳过）
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

  // 监听安装进度事件
  useEffect(() => {
    const unlisten = listen<{ tool: string; progress: number; done: boolean }>(
      "install_progress",
      (event) => {
        const { tool, progress, done } = event.payload;
        setInstallProgress((s) => ({ ...s, [tool]: progress }));
        setInstallModal((prev) => {
          if (!prev || prev.tool !== tool) return prev;
          const phase = progress <= 30 ? "卸载" : "安装";
          const line = done
            ? `✓ 安装流程结束，正在检测版本...`
            : `${phase}中... ${progress}%`;
          const lines = [...prev.logLines, line];
          return { ...prev, logLines: lines, progress, done };
        });
        if (done) {
          setTimeout(async () => {
            await fetchAll();
            setInstalling((s) => ({ ...s, [tool]: false }));
            setInstallProgress((s) => {
              const next = { ...s };
              delete next[tool];
              return next;
            });
            setInstallModal(null);
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
    const toolName = ALL_TOOLS.find((it) => it.key === toolKey)?.name || toolKey;
    setInstallModal({ tool: toolName, logLines: [`开始卸载 ${toolName}...`], progress: 0, done: false });
    try {
      await invoke("install_tool", { tool: toolKey });
    } catch (e: any) {
      console.error("install error:", e);
      setInstallModal((prev) => prev ? {
        ...prev, logLines: [...prev.logLines, `错误: ${e?.toString() || "未知错误"}`], done: true
      } : prev);
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
        <span className="text-[11px] text-muted-foreground flex items-center gap-1">
          <Loader2 className="w-3 h-3 animate-spin" />
          检测中
        </span>
      );
    }
    const version = versions[item.key];
    const installed = version !== undefined && version !== null;
    if (!installed) {
      return <span className="text-[11px] text-muted-foreground">未检测到</span>;
    }
    const display = version.length > 15 ? version.slice(0, 14) + "…" : version;
    return (
      <span className="text-[11px] text-green-500 font-mono whitespace-nowrap overflow-hidden text-ellipsis leading-tight" title={version}>
        {display}
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
      <div key={item.key} className="flex items-center gap-2 py-1.5">
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
        <div className="flex flex-col min-w-0 flex-1 overflow-hidden gap-0.5">
          <span className="text-[13px] font-medium text-foreground leading-tight whitespace-nowrap overflow-hidden text-overflow-ellipsis">
            {item.name}
          </span>
          {renderStatusText(item)}
        </div>
        <div className="flex flex-row gap-1 shrink-0">
          {isNpm ? (
            <>
              {installed && npmAvail.checked ? (
                <div className="flex items-center justify-center flex-1">
                  <span className={`h-6 flex items-center justify-center text-[11px] px-1.5 rounded-md border font-medium whitespace-nowrap ${
                    npmAvail.available
                      ? "bg-green-500/10 border-green-500/30 text-green-500"
                      : "bg-red-500/10 border-red-500/30 text-red-500"
                  }`}>
                    {npmAvail.available ? "可用" : "不可用"}
                  </span>
                </div>
              ) : (
                <div className="flex items-center justify-center flex-1">
                  <span className="h-6 flex items-center justify-center text-[11px] px-1.5 text-muted-foreground whitespace-nowrap">
                    检测中...
                  </span>
                </div>
              )}
              {installed && npmAvail.checked && !npmAvail.available && (
                <Button
                  variant="outline"
                  size="sm"
                  className="h-6 text-[11px] px-1.5 py-[3px] text-orange-500 border-orange-500/30 hover:bg-orange-500/10 whitespace-nowrap gap-0"
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
              <Button
                variant="outline"
                size="sm"
                className="h-6 text-[11px] px-1.5 py-[3px] whitespace-nowrap gap-0"
                disabled={!!isInstalling}
                onClick={() => handleInstall(item.key)}
              >
                {isInstalling ? (
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                ) : (
                  <Terminal className="w-3.5 h-3.5" />
                )}
                {isInstalling
                  ? (progress !== undefined
                      ? (progress <= 30 ? `卸载 ${progress}%` : `安装 ${progress}%`)
                      : "处理中...")
                  : (installed ? "重装" : "安装")}
              </Button>
              <Button
                variant="outline"
                size="sm"
                className="h-6 text-[11px] px-1.5 py-[3px] whitespace-nowrap gap-0"
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
    <div className="relative flex items-center justify-center min-h-screen w-screen bg-background px-4 py-4">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4, ease: "easeOut" }}
        className="glass-card w-full max-w-5xl flex flex-col p-5"
      >
        {/* Header */}
        <div className="flex flex-col items-center gap-2 mb-3">
          <div className="w-10 h-10 rounded-lg bg-blue-600 flex items-center justify-center">
            <Monitor className="w-5 h-5 text-white" />
          </div>
          <h1 className="text-xl font-bold text-foreground">环境检测</h1>
          <p className="text-sm text-muted-foreground flex items-center gap-1.5">
            检测开发环境依赖的安装状态
            {!loading && (
              <span>
                （{installedCount}/{totalCount} 已安装）
              </span>
            )}
            <button
              onClick={() => fetchAll()}
              disabled={loading}
              className="inline-flex items-center justify-center w-5 h-5 rounded hover:bg-muted-foreground/10 transition-colors disabled:opacity-40"
              title="重新检测"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${loading ? "animate-spin" : ""}`} />
            </button>
          </p>
        </div>

        {/* Tool rows */}
        <div className="flex flex-col rounded-lg border border-border/60 bg-muted/30 p-4">
          <div className="flex flex-col">
            {ALL_TOOLS.map(renderToolRow)}
          </div>
        </div>

        {/* Status */}
        <div className="flex items-center gap-3 justify-center mt-3">
          {!loading && installedCount < totalCount && (
            <span className="text-[11px] text-muted-foreground flex items-center gap-1">
              <AlertCircle className="w-3 h-3 text-yellow-500" />
              部分工具未安装，可点击"安装"或"官网"按钮
            </span>
          )}
          {!loading && installedCount === totalCount && (
            <span className="text-[11px] text-green-500 flex items-center gap-1">
              <CheckCircle2 className="w-3 h-3" />
              所有工具已安装
            </span>
          )}
        </div>

        {/* Batch install button */}
        <Button
          variant="default"
          size="sm"
          className="w-full mt-2 h-8 text-[12px]"
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
        <div className="mt-2 rounded-lg border border-border/60 bg-muted/30 p-3">
          <button
            type="button"
            onClick={() => {
              if (!showDebug && !debugOutput) {
                handleDebug();
              } else {
                setShowDebug((v) => !v);
              }
            }}
            className="flex w-full items-center gap-2 text-[12px] font-medium text-foreground transition-colors hover:text-primary"
          >
            <ChevronDown
              className={`h-3.5 w-3.5 transition-transform ${showDebug ? "" : "-rotate-90"}`}
            />
            <Stethoscope className="h-3.5 w-3.5" />
            安装环境诊断
            {debugLoading && <Loader2 className="h-3 w-3 animate-spin text-muted-foreground" />}
          </button>

          {showDebug && debugOutput && (
            <div className="mt-2 rounded border border-border/60 bg-background/80 p-3">
              <pre className="text-[11px] font-mono text-foreground whitespace-pre-wrap break-all leading-relaxed max-h-48 overflow-auto">
                {debugOutput}
              </pre>
            </div>
          )}
        </div>

        {/* Enter button */}
        <Button onClick={onDone} className="w-full mt-3 h-9 text-sm">
          完成，进入 DevClaw
          <ArrowRight className="w-4 h-4 ml-2" />
        </Button>
      </motion.div>

      {/* 安装弹窗 */}
      {installModal && (
        <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
          <div className="flex flex-col w-[600px] h-[400px] rounded-xl border border-border/60 bg-background shadow-2xl overflow-hidden">
            <div className="flex items-center justify-between px-5 py-3 border-b border-border/40">
              <span className="text-sm font-medium text-foreground">
                {installModal.done && installModal.tool !== "历史日志" ? "安装完成" : `正在安装 ${installModal.tool}`}
              </span>
              <button
                disabled={!installModal.done}
                onClick={() => setInstallModal(null)}
                className="w-6 h-6 flex items-center justify-center rounded hover:bg-muted-foreground/10 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="flex-1 overflow-y-auto px-5 py-3 bg-black/80">
              <pre className="text-[11px] text-green-400 whitespace-pre-wrap font-mono leading-relaxed">
                {installModal.logLines.join("\n")}
              </pre>
            </div>

            <div className="px-5 py-3 border-t border-border/40 flex flex-col gap-2">
              {installModal.tool !== "历史日志" && (
                <div className="w-full h-2 rounded-full bg-muted overflow-hidden">
                  <div
                    className="h-full rounded-full bg-blue-500 transition-all duration-300"
                    style={{ width: `${installModal.progress}%` }}
                  />
                </div>
              )}
              <div className="flex items-center justify-between">
                <span className="text-[11px] text-muted-foreground">
                  {installModal.tool !== "历史日志"
                    ? (installModal.done ? "安装完成，正在检测版本..." : `${installModal.progress}%`)
                    : "历史安装日志"}
                </span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
