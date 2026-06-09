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

export interface UpdateProgressEvent {
  event: "Started" | "Progress" | "Finished";
  total?: number;
  downloaded?: number;
}

export interface UpdateHandle {
  version: string;
  notes?: string;
  date?: string;
  downloadAndInstall: (
    onProgress?: (e: UpdateProgressEvent) => void,
  ) => Promise<void>;
}

export interface CheckOptions {
  timeout?: number;
}

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

  // 优先使用 tauri-plugin-updater
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check({ timeout: opts.timeout ?? 30000 });

    if (!update) {
      return { status: "up-to-date" };
    }

    const info: UpdateInfo = {
      currentVersion,
      availableVersion: update.version ?? "",
      notes: (update as any).body ?? undefined,
      pubDate: (update as any).date ?? undefined,
    };

    const handle: UpdateHandle = {
      version: update.version ?? "",
      notes: info.notes,
      date: info.pubDate,
      async downloadAndInstall(onProgress?: (e: UpdateProgressEvent) => void) {
        let totalDownloaded = 0;
        await update.downloadAndInstall((event) => {
          if (!onProgress) return;
          switch (event.event) {
            case "Started":
              totalDownloaded = 0;
              onProgress({
                event: "Started",
                total: (event.data as any)?.contentLength ?? 0,
                downloaded: 0,
              });
              break;
            case "Progress":
              totalDownloaded += (event.data as any)?.chunkLength ?? 0;
              onProgress({
                event: "Progress",
                downloaded: totalDownloaded,
              });
              break;
            case "Finished":
              onProgress({ event: "Finished" });
              break;
          }
        });
      },
    };

    return { status: "available", info, update: handle };
  } catch {
    // 插件不可用 → 回退 GitHub API
  }

  return await checkViaGitHubApi(currentVersion);
}

async function checkViaGitHubApi(
  currentVersion: string,
): Promise<
  | { status: "up-to-date" }
  | { status: "available"; info: UpdateInfo; update: UpdateHandle }
> {
  const GITHUB_REPO = "AnXiYiZhi/DevClaw";
  const GITHUB_API_URL = `https://api.github.com/repos/${GITHUB_REPO}/releases`;

  const resp = await fetch(GITHUB_API_URL, {
    headers: { Accept: "application/vnd.github.v3+json" },
  });
  if (!resp.ok) return { status: "up-to-date" };

  const releases: Array<{
    tag_name?: string;
    draft?: boolean;
    prerelease?: boolean;
    body?: string;
    published_at?: string;
  }> = await resp.json();

  const data =
    releases.find((r) => !r.draft && !r.prerelease) ??
    releases.find((r) => !r.draft);
  if (!data?.tag_name) return { status: "up-to-date" };

  const latestVersion = data.tag_name.replace(/^v/, "");
  if (!isNewer(latestVersion, currentVersion)) {
    return { status: "up-to-date" };
  }

  const info: UpdateInfo = {
    currentVersion,
    availableVersion: latestVersion,
    notes: data.body ?? undefined,
    pubDate: data.published_at ?? undefined,
  };

  return {
    status: "available",
    info,
    update: {
      version: latestVersion,
      notes: info.notes,
      date: info.pubDate,
      async downloadAndInstall() {
        window.open(
          `https://github.com/${GITHUB_REPO}/releases/latest`,
          "_blank",
        );
      },
    },
  };
}

export async function relaunchApp(): Promise<void> {
  const { relaunch } = await import("@tauri-apps/plugin-process");
  await relaunch();
}
