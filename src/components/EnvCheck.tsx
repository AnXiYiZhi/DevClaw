import { useEffect, useState, useCallback } from "react";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import {
  Loader2,
  CheckCircle,
  XCircle,
  ExternalLink,
  ArrowRight,
  Monitor,
  Terminal,
  RefreshCw,
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

const BASIC_TOOLS: EnvItem[] = [
  { key: "vscode", name: "VS Code", iconUrl: "https://cdn.simpleicons.org/visualstudiocode", fallbackLetter: "V", fallbackColor: "#007ACC", downloadUrl: "https://code.visualstudio.com" },
  { key: "chrome", name: "Chrome", iconUrl: "https://cdn.simpleicons.org/googlechrome", fallbackLetter: "C", fallbackColor: "#4285F4", downloadUrl: "https://chrome.google.com" },
];

const AI_TOOLS: EnvItem[] = [
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

  const installedCount = Object.values(versions).filter(Boolean).length;
  const totalCount = BASIC_ENV.length + BASIC_TOOLS.length + AI_TOOLS.length;

  const renderStatusText = (item: EnvItem) => {
    if (loading) {
      return (
        <span className="text-[12px] text-muted-foreground flex items-center gap-1">
          <Loader2 className="w-3 h-3 animate-spin" />
          检测中
        </span>
      );
    }
    const version = versions[item.key];
    const installed = version !== undefined && version !== null;
    if (!installed) {
      return <span className="text-[12px] text-muted-foreground">未检测到</span>;
    }
    return (
      <span className="flex items-center gap-1.5">
        <span className="text-[12px] text-green-500 font-mono">{version}</span>
        {item.key === "npm" && npmAvail.checked && (
          <span className={`text-[10px] ${npmAvail.available ? "text-green-500" : "text-red-500"}`}>
            {npmAvail.available ? "可用" : "不可用"}
          </span>
        )}
      </span>
    );
  };

  const renderGroup = (title: string, items: EnvItem[]) => (
    <div className="flex flex-col rounded-xl border border-border/60 bg-muted/30 p-4">
      <h3 className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-3">
        {title}
      </h3>
      <div className="flex flex-col">
        {items.map((item, i) => {
          const version = loading ? undefined : versions[item.key];
          const installed = version !== undefined && version !== null;
          const isInstalling = installing[item.key];
          const msg = installMsg[item.key];
          return (
            <div key={item.key}>
              {i > 0 && <div className="border-t border-border/40" />}
              <div className="flex items-center gap-3" style={{ height: 72 }}>
                <ToolIcon item={item} />
                <div className="flex flex-col min-w-0 flex-1 gap-0.5">
                  <span className="text-[14px] font-medium text-foreground truncate">
                    {item.name}
                  </span>
                  {renderStatusText(item)}
                </div>
                <div className="flex flex-col gap-1 shrink-0" style={{ width: 88 }}>
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-7 text-[11px] w-full px-2"
                    disabled={!!isInstalling}
                    onClick={() => handleInstall(item.key)}
                  >
                    {isInstalling ? (
                      <Loader2 className="w-3 h-3 animate-spin mr-1" />
                    ) : (
                      <Terminal className="w-3 h-3 mr-1" />
                    )}
                    {isInstalling ? "安装中..." : (installed ? "重装" : "安装")}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-7 text-[11px] w-full px-2"
                    onClick={() => openUrl(item.downloadUrl)}
                  >
                    <ExternalLink className="w-3 h-3 mr-1" />
                    官网
                  </Button>
                </div>
              </div>
              {msg && (
                <div className={`text-[10px] px-1 py-0.5 rounded mb-1 ${msg.includes("成功") ? "text-green-600 bg-green-500/10" : "text-red-500 bg-red-500/10"}`}>
                  {msg.split("\n")[0]}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );

  return (
    <div className="flex items-center justify-center h-screen w-screen bg-background px-6">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4, ease: "easeOut" }}
        className="glass-card w-full max-w-3xl flex flex-col p-6"
      >
        <div className="flex flex-col items-center gap-3 mb-3">
          <div className="w-14 h-14 rounded-xl bg-blue-600 flex items-center justify-center">
            <Monitor className="w-7 h-7 text-white" />
          </div>
          <h1 className="text-2xl font-bold text-foreground">环境检测</h1>
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

        <div className="grid grid-cols-3 gap-4">
          {renderGroup("基础环境", BASIC_ENV)}
          {renderGroup("基础工具", BASIC_TOOLS)}
          {renderGroup("AI 工具", AI_TOOLS)}
        </div>

        <div className="flex items-center gap-3 justify-center mt-3">
          {!loading && installedCount < totalCount && (
            <span className="text-xs text-muted-foreground flex items-center gap-1">
              <XCircle className="w-3 h-3 text-yellow-500" />
              部分工具未安装，可点击"安装"或"官网"按钮
            </span>
          )}
          {!loading && installedCount === totalCount && (
            <span className="text-xs text-green-500 flex items-center gap-1">
              <CheckCircle className="w-3 h-3" />
              所有工具已安装
            </span>
          )}
        </div>

        {/* 临时调试按钮 */}
        <div className="flex flex-col gap-2 mt-2">
          <Button
            variant="destructive"
            size="sm"
            className="w-full"
            disabled={debugLoading}
            onClick={async () => {
              setDebugLoading(true);
              try {
                const result = await invoke<string>("debug_env");
                setDebugOutput(result);
              } catch (e: any) {
                setDebugOutput(`调用失败: ${e?.toString()}`);
              } finally {
                setDebugLoading(false);
              }
            }}
          >
            {debugLoading ? <Loader2 className="w-4 h-4 animate-spin mr-2" /> : null}
            [调试] 运行 debug_env
          </Button>
          {debugOutput && (
            <pre className="text-[10px] bg-black/80 text-green-400 p-3 rounded-lg overflow-auto max-h-60 whitespace-pre-wrap font-mono">
              {debugOutput}
            </pre>
          )}
        </div>

        <Button onClick={onDone} size="lg" className="w-full mt-3">
          完成，进入 DevClaw
          <ArrowRight className="w-4 h-4 ml-2" />
        </Button>
      </motion.div>
    </div>
  );
}
