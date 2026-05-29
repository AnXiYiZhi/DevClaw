import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { Shield, Check, Copy, CheckCircle, XCircle, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { toast } from "sonner";

interface ActivationPageProps {
  onActivated: () => void;
}

export function ActivationPage({ onActivated }: ActivationPageProps) {
  const [deviceId, setDeviceId] = useState("");
  const [activationCode, setActivationCode] = useState("");
  const [status, setStatus] = useState<"idle" | "loading" | "success" | "error">("idle");
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    invoke<string>("get_device_id").then(setDeviceId).catch(() => {});
  }, []);

  const copyDeviceId = async () => {
    try {
      await navigator.clipboard.writeText(deviceId);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // 非安全上下文回退
      const ta = document.createElement("textarea");
      ta.value = deviceId;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    }
  };

  const handleActivate = async () => {
    if (!activationCode.trim()) return;
    setStatus("loading");
    setError("");

    try {
      const result = await invoke<{ ok: boolean; error?: string }>(
        "verify_license",
        { license: activationCode.trim() },
      );

      if (result.ok) {
        setStatus("success");
        toast.success("激活成功");
        setTimeout(onActivated, 800);
      } else {
        setStatus("error");
        setError(result.error || "激活码验证失败");
      }
    } catch (e: any) {
      setStatus("error");
      setError(e?.toString() || "验证过程出错");
    }
  };

  return (
    <div className="flex items-center justify-center h-screen w-screen bg-background">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4, ease: "easeOut" }}
        className="glass-card p-8 w-full max-w-lg flex flex-col gap-6"
      >
        <div className="flex flex-col items-center gap-3">
          <div className="w-14 h-14 rounded-xl bg-blue-600 flex items-center justify-center">
            <Shield className="w-7 h-7 text-white" />
          </div>
          <h1 className="text-2xl font-bold text-foreground">DevClaw</h1>
          <p className="text-sm text-muted-foreground">请输入授权密钥以继续使用</p>
        </div>

        <div className="flex flex-col gap-2">
          <label className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
            设备 ID
          </label>
          <div className="flex gap-2">
            <code className="flex-1 h-9 rounded-md bg-muted px-3 flex items-center text-xs font-mono text-muted-foreground overflow-x-auto select-all">
              {deviceId || "获取中..."}
            </code>
            <Button
              variant="outline"
              size="icon"
              onClick={copyDeviceId}
              disabled={!deviceId}
              title="复制设备 ID"
            >
              {copied ? (
                <Check className="w-4 h-4 text-green-500" />
              ) : (
                <Copy className="w-4 h-4" />
              )}
            </Button>
          </div>
        </div>

        <div className="flex flex-col gap-2">
          <label className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
            授权密钥
          </label>
          <Input
            type="text"
            value={activationCode}
            onChange={(e) => setActivationCode(e.target.value.toUpperCase())}
            placeholder="CCS-XXXX-XXXX-XXXX"
            disabled={status === "loading"}
            className="font-mono text-sm tracking-wider"
          />
          <p className="text-xs text-muted-foreground">
            格式: CCS-XXXX-XXXX-XXXX（字母大写+数字）
          </p>
        </div>

        <Button
          onClick={handleActivate}
          disabled={!activationCode.trim() || status === "loading"}
          size="lg"
          className="w-full"
        >
          {status === "loading" ? (
            <>
              <Loader2 className="w-4 h-4 animate-spin" />
              验证中...
            </>
          ) : (
            "验证激活"
          )}
        </Button>

        {status === "success" && (
          <motion.div
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
            className="flex items-center gap-2 justify-center text-green-500"
          >
            <CheckCircle className="w-4 h-4" />
            <span className="text-sm">激活成功，正在进入应用...</span>
          </motion.div>
        )}

        {status === "error" && (
          <motion.div
            initial={{ opacity: 0, y: 4 }}
            animate={{ opacity: 1, y: 0 }}
            className="flex items-start gap-2 p-3 rounded-lg bg-red-500/10 border border-red-500/20"
          >
            <XCircle className="w-4 h-4 text-red-500 mt-0.5 shrink-0" />
            <span className="text-sm text-red-500">{error}</span>
          </motion.div>
        )}
      </motion.div>
    </div>
  );
}
