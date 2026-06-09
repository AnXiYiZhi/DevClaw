import { useEffect, useState, useCallback } from "react";
import { motion } from "framer-motion";
import {
  isWindows,
  isLinux,
  DRAG_REGION_ATTR,
  DRAG_REGION_STYLE,
} from "@/lib/platform";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Loader2,
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
  Settings,
  FileText,
  FolderOpen,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import iconNodejs from "@/assets/icons/nodejs.svg";
import iconNpm from "@/assets/icons/npm.svg";
import iconGit from "@/assets/icons/git.svg";
import iconPython from "@/assets/icons/python.svg";
import iconVscode from "@/assets/icons/vscode.svg";
import iconChrome from "@/assets/icons/chrome.svg";
// Claude SVG embedded as data URL (file import may fail in Tauri webview)
const CLAUDE_ICON_DATA_URL =
  "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZmlsbD0iI0Q5Nzc1NyIgZD0ibTQuNzE0NCAxNS45NTU1IDQuNzE3NC0yLjY0NzEuMDc5LS4yMzA3LS4wNzktLjEyNzVoLS4yMzA3bC0uNzg5My0uMDQ4Ni0yLjY5NTYtLjA3MjktMi4zMzc1LS4wOTcxLTIuMjY0Ni0uMTIxNC0uNTcwNy0uMTIxNS0uNTM0My0uNzA0Mi4wNTQ2LS4zNTIyLjQ3OTctLjMyMTguNjg2LjA2MDggMS41MTc5LjEwMzIgMi4yNzY3LjE1NzggMS42NTE0LjA5NzIgMi40NDY4LjI1NWguMzg4NmwuMDU0Ni0uMTU3OS0uMTMzNi0uMDk3MS0uMTAzMi0uMDk3Mkw2Ljk3MyA5LjgzNTZsLTIuNTUtMS42ODc5LTEuMzM1Ni0uOTcxNC0uNzIyNS0uNDkxOC0uMzY0My0uNDYxNC0uMTU3OC0xLjAwNzguNjU1Ny0uNzIyNS44ODAzLjA2MDcuMjI0Ni4wNjA3Ljg5MjUuNjg2IDEuOTA2NCAxLjQ3NTQgMi40ODkzIDEuODMzNi4zNjQzLjMwMzUuMTQ1Ny0uMTAzMi4wMTgyLS4wNzI4LS4xNjQtLjI3MzMtMS4zNTM5LTIuNDQ2Ny0xLjQ0NS0yLjQ4OTMtLjY0MzUtMS4wMzItLjE3LS42MTk0Yy0uMDYwNy0uMjU1LS4xMDMyLS40Njc0LS4xMDMyLS43Mjg1TDYuMjg3LjEzMzUgNi42OTk3IDBsLjk5NTcuMTMzNi40MTkuMzY0Mi42MTkyIDEuNDE0NyAxLjAwMTggMi4yMjgyIDEuNTU0MyAzLjAyOTYuNDU1My44OTg1LjI0MjkuODMxOC4wOTEuMjU1aC4xNTc5di0uMTQ1N2wuMTI3NS0xLjcwNi4yMzY4LTIuMDk0Ny4yMzA3LTIuNjk1Ny4wNzg5LS43NTg5LjM3NjQtLjkxMDcuNzQ2OC0uNDkxOC41ODI4LjI3OTMuNDc5Ny42ODYtLjA2NjguNDQzMy0uMjg1MyAxLjg1MTctLjU1ODYgMi45MDIxLS4zNjQzIDEuOTQyOWguMjEyNWwuMjQyOS0uMjQyOS45ODM1LTEuMzA1MyAxLjY1MTQtMi4wNjQzLjcyODYtLjgxOTYuODUtLjkwNDYuNTQ2NC0uNDMxMWgxLjAzMjFsLjc1OSAxLjEyOTMtLjM0IDEuMTY1Ny0xLjA2MjUgMS4zNDc4LS44ODA0IDEuMTQxNC0xLjI2MjggMS43LS43ODkzIDEuMzYuMDcyOS4xMDkzLjE4ODItLjAxODMgMi44NTM1LS42MDcgMS41NDIxLS4yNzk0IDEuODM5Ni0uMzE1Ny44MzE4LjM4ODYuMDkxLjM5NDYtLjMyNzguODA3NS0xLjk2Ny40ODU3LTIuMzA3Mi40NjE0LTMuNDM2NC44MTM2LS4wNDI1LjAzMDQuMDQ4Ni4wNjA3IDEuNTQ4Mi4xNDU3LjY2MTguMDM2NGgxLjYyMWwzLjAxNzUuMjI0Ny43ODkyLjUyMi40NzM2LjYzNzYtLjA3OS40ODU3LTEuMjE0Mi42MTkzLTEuNjM5My0uMzg4Ni0zLjgyNS0uOTEwNy0xLjMxMTMtLjMyNzloLS4xODIydi4xMDkzbDEuMDkyOSAxLjA2ODYgMi4wMDM1IDEuODA5MiAyLjUwNzUgMi4zMzE0LjEyNzUuNTc2OC0uMzIxOC40NTU0LS4zNC0uMDQ4Ni0yLjIwMzktMS42NTc1LS44NS0uNzQ2OC0xLjkyNDYtMS42MjFoLS4xMjc1di4xN2wuNDQzMi42NDk2IDIuMzQzNiAzLjUyMTQuMTIxNCAxLjA4MDctLjE3LjM1MjEtLjYwNzEuMjEyNS0uNjY3OS0uMTIxNC0xLjM3MjEtMS45MjQ2TDE0LjM4IDE3Ljk1OWwtMS4xNDE0LTEuOTQyOC0uMTM5Ny4wNzktLjY3NCA3LjI1NTItLjMxNTYuMzcwMy0uNzI4Ni4yNzkzLS42MDcxLS40NjE0LS4zMjE4LS43NDY4LjMyMTgtMS40NzUzLjM4ODYtMS45MjQ2LjMxNTctMS41My4yODUzLTEuOTAwNC4xNy0uNjMxNC0uMDEyMS0uMDQyNS0uMTM5Ny4wMTgyLTEuNDMyOCAxLjk2NzItMi4xNzk2IDIuOTQ0Ni0xLjcyNDMgMS44NDU2LS40MTI4LjE2NC0uNzE2NC0uMzcwNC4wNjY3LS42NjE4LjQwMDgtLjU4ODkgMi4zODYtMy4wMzU3IDEuNDM4OS0xLjg4Mi45MjktMS4wODY4LS4wMDYyLS4xNTc5aC0uMDU0NmwtNi4zMzg1IDQuMTE2NC0xLjEyOTMuMTQ1Ny0uNDg1Ny0uNDU1NC4wNjA4LS43NDY3LjIzMDctLjI0MjkgMS45MDY0LTEuMzExNFoiLz48L3N2Zz4=";

interface EnvCheckProps {
  onDone: () => void;
  onNavigate?: (view: string) => void;
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
  {
    key: "nodejs",
    name: "Node.js",
    iconUrl: iconNodejs,
    fallbackLetter: "N",
    fallbackColor: "#339933",
    downloadUrl: "https://nodejs.org/en/download",
  },
  {
    key: "npm",
    name: "npm",
    iconUrl: iconNpm,
    fallbackLetter: "n",
    fallbackColor: "#CB3837",
    downloadUrl: "https://nodejs.org/en/download",
  },
  {
    key: "git",
    name: "Git",
    iconUrl: iconGit,
    fallbackLetter: "G",
    fallbackColor: "#F05032",
    downloadUrl: "https://git-scm.com/downloads",
  },
  {
    key: "python",
    name: "Python",
    iconUrl: iconPython,
    fallbackLetter: "P",
    fallbackColor: "#3776AB",
    downloadUrl: "https://www.python.org/downloads/",
  },
];

const TOOLS: EnvItem[] = [
  {
    key: "vscode",
    name: "VS Code",
    iconUrl: iconVscode,
    fallbackLetter: "V",
    fallbackColor: "#007ACC",
    downloadUrl: "https://code.visualstudio.com/Download",
  },
  {
    key: "chrome",
    name: "Chrome",
    iconUrl: iconChrome,
    fallbackLetter: "C",
    fallbackColor: "#4285F4",
    downloadUrl: "https://www.google.com/chrome/",
  },
  {
    key: "claude",
    name: "Claude CLI",
    iconUrl: CLAUDE_ICON_DATA_URL,
    fallbackLetter: "C",
    fallbackColor: "#D97757",
    downloadUrl: "https://claude.ai/download",
  },
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

export function EnvCheck({ onDone, onNavigate }: EnvCheckProps) {
  const [versions, setVersions] = useState<Record<string, string | null>>({});
  const [loading, setLoading] = useState(true);
  const [npmAvail, setNpmAvail] = useState<NpmAvailability>({
    available: false,
    checked: false,
  });
  const [installing, setInstalling] = useState<Record<string, boolean>>({});
  const [installProgress, setInstallProgress] = useState<
    Record<string, number>
  >({});
  const [fixInProgress, setFixInProgress] = useState(false);
  const [selectedTools, setSelectedTools] = useState<Record<string, boolean>>(
    {},
  );
  const [batchInstalling, setBatchInstalling] = useState(false);
  const [debugOutput, setDebugOutput] = useState<string | null>(null);
  const [debugLoading, setDebugLoading] = useState(false);
  const [showDebug, setShowDebug] = useState(false);
  const [logsContent, setLogsContent] = useState<string | null>(null);
  const [logsLoading, setLogsLoading] = useState(false);
  const [showLogs, setShowLogs] = useState(false);

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
      },
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
    const INSTALL_ORDER = [
      "nodejs",
      "git",
      "python",
      "vscode",
      "chrome",
      "claude",
    ];
    const tools = INSTALL_ORDER.filter((k) => selectedTools[k]);
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

  const handleViewLogs = async () => {
    if (!showLogs && !logsContent) {
      setLogsLoading(true);
      try {
        const content = await invoke<string>("get_logs_content");
        setLogsContent(content);
        setShowLogs(true);
      } catch (e: any) {
        setLogsContent(`获取失败: ${e?.toString()}`);
        setShowLogs(true);
      } finally {
        setLogsLoading(false);
      }
    } else {
      setShowLogs((v) => !v);
    }
  };

  const handleOpenLogsDir = async () => {
    try {
      await invoke("open_logs_dir");
    } catch {
      // ignore
    }
  };

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
      <span
        className="text-xs text-green-500 font-mono truncate"
        title={version}
      >
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
            onClick={() =>
              setSelectedTools((s) => ({ ...s, [item.key]: !s[item.key] }))
            }
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
                <span
                  className={`h-7 flex items-center justify-center text-xs px-2 rounded-md border font-medium whitespace-nowrap ${
                    npmAvail.available
                      ? "bg-green-500/10 border-green-500/30 text-green-500"
                      : "bg-red-500/10 border-red-500/30 text-red-500"
                  }`}
                >
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
              {item.key === "claude" && onNavigate ? (
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 text-xs px-2 whitespace-nowrap gap-1"
                  onClick={() => onNavigate("claudeProxy")}
                >
                  <Settings className="w-3.5 h-3.5" />
                  配置
                </Button>
              ) : (
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 text-xs px-2 whitespace-nowrap gap-1"
                  onClick={() => openUrl(item.downloadUrl)}
                >
                  <ExternalLink className="w-3.5 h-3.5" />
                  官网
                </Button>
              )}
            </>
          )}
        </div>
      </div>
    );
  };

  return (
    <div className="relative w-full h-full flex flex-col bg-background">
      {/* Drag bar for macOS titlebar */}
      {!isWindows() && !isLinux() && (
        <div
          {...DRAG_REGION_ATTR}
          style={{ ...DRAG_REGION_STYLE, height: 28 } as any}
          className="shrink-0"
        />
      )}
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
              <h1 className="text-lg font-semibold text-foreground">
                环境检测
              </h1>
              {!loading && (
                <p className="text-sm text-muted-foreground">
                  {installedCount}/{totalCount} 已安装
                </p>
              )}
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => fetchAll()}
              disabled={loading}
              className="h-8 gap-1.5 text-xs shrink-0"
            >
              <RefreshCw
                className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`}
              />
              重新检测
            </Button>
          </div>

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
            disabled={
              batchInstalling ||
              loading ||
              !Object.values(selectedTools).some(Boolean)
            }
            onClick={handleBatchInstall}
          >
            {batchInstalling ? (
              <Loader2 className="w-4 h-4 animate-spin mr-1.5" />
            ) : (
              <Download className="w-4 h-4 mr-1.5" />
            )}
            一键安装（{Object.values(selectedTools).filter(Boolean).length} 项）
          </Button>

          {/* Enter button */}
          <Button onClick={onDone} size="lg" className="w-full">
            完成，进入 DevClaw
            <ArrowRight className="w-4 h-4 ml-2" />
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
              {debugLoading && (
                <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground" />
              )}
            </button>

            {showDebug && debugOutput && (
              <div className="mt-3 rounded-lg border border-border/60 bg-background/80 p-4">
                <pre className="text-xs font-mono text-foreground whitespace-pre-wrap break-all leading-relaxed max-h-96 overflow-auto">
                  {debugOutput}
                </pre>
              </div>
            )}
          </div>

          {/* Logs section */}
          <div className="rounded-xl border border-border bg-gradient-to-br from-card/80 to-card/40 p-4 shadow-sm">
            <div className="flex items-center justify-between">
              <button
                type="button"
                onClick={handleViewLogs}
                className="flex items-center gap-2 text-sm font-medium text-foreground transition-colors hover:text-primary"
              >
                <ChevronDown
                  className={`h-4 w-4 transition-transform ${showLogs ? "" : "-rotate-90"}`}
                />
                <FileText className="h-4 w-4" />
                安装日志
                {logsLoading && (
                  <Loader2 className="h-3.5 w-3.5 animate-spin text-muted-foreground" />
                )}
              </button>
              <Button
                variant="outline"
                size="sm"
                className="h-7 text-xs px-2 gap-1"
                onClick={handleOpenLogsDir}
              >
                <FolderOpen className="w-3.5 h-3.5" />
                日志
              </Button>
            </div>

            {showLogs && logsContent && (
              <div className="mt-3 rounded-lg border border-border/60 bg-background/80 p-4">
                <pre className="text-xs font-mono text-foreground whitespace-pre-wrap break-all leading-relaxed max-h-[600px] overflow-auto">
                  {logsContent}
                </pre>
              </div>
            )}
          </div>
        </div>
      </motion.div>
    </div>
  );
}
