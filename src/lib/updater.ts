import { getVersion } from "@tauri-apps/api/app";

export type UpdateChannel = "stable" | "beta";

export type UpdaterPhase =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "restarting"
  | "upToDate"
  | "error";

export interface UpdateInfo {
  currentVersion: string;
  availableVersion: string;
  notes?: string;
  pubDate?: string;
}

export interface UpdateHandle {
  version: string;
  notes?: string;
  date?: string;
  downloadAndInstall: (
    onProgress?: (e: UpdateProgressEvent) => void,
  ) => Promise<void>;
  download?: () => Promise<void>;
  install?: () => Promise<void>;
}

export interface UpdateProgressEvent {
  event: "Started" | "Progress" | "Finished";
  total?: number;
  downloaded?: number;
}

export interface CheckOptions {
  timeout?: number;
  channel?: UpdateChannel;
}

const GITHUB_REPO = "AnXiYiZhi/DevCLaw";
const GITHUB_API_URL = `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`;

export async function getCurrentVersion(): Promise<string> {
  try {
    return await getVersion();
  } catch {
    return "";
  }
}

function parseVersion(v: string): number[] {
  return v.replace(/^v/, "").split(".").map(Number);
}

function isNewer(latest: string, current: string): boolean {
  const l = parseVersion(latest);
  const c = parseVersion(current);
  for (let i = 0; i < Math.max(l.length, c.length); i++) {
    const li = l[i] ?? 0;
    const ci = c[i] ?? 0;
    if (li > ci) return true;
    if (li < ci) return false;
  }
  return false;
}

export async function checkForUpdate(
  opts: CheckOptions = {},
): Promise<
  | { status: "up-to-date" }
  | { status: "available"; info: UpdateInfo; update: UpdateHandle }
> {
  const currentVersion = await getCurrentVersion();

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), opts.timeout ?? 30000);

  try {
    const resp = await fetch(GITHUB_API_URL, {
      signal: controller.signal,
      headers: { Accept: "application/vnd.github.v3+json" },
    });

    if (!resp.ok) {
      throw new Error(`GitHub API returned ${resp.status}`);
    }

    const data = await resp.json();
    const tagName: string = data.tag_name ?? "";
    const latestVersion = tagName.replace(/^v/, "");

    if (!latestVersion || !isNewer(latestVersion, currentVersion)) {
      return { status: "up-to-date" };
    }

    const info: UpdateInfo = {
      currentVersion,
      availableVersion: latestVersion,
      notes: data.body ?? undefined,
      pubDate: data.published_at ?? undefined,
    };

    const update: UpdateHandle = {
      version: latestVersion,
      notes: info.notes,
      date: info.pubDate,
      async downloadAndInstall() {
        // No auto-download — direct user to website
        window.open("https://devclaw.cc.cd", "_blank");
      },
    };

    return { status: "available", info, update };
  } finally {
    clearTimeout(timeout);
  }
}

export async function relaunchApp(): Promise<void> {
  const { relaunch } = await import("@tauri-apps/plugin-process");
  await relaunch();
}
