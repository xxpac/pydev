import { locale } from "svelte-i18n";

import { api } from "./api";
import type { Config, EnvStatus, LogLine, OsInfo, Stage } from "./types";

const MAX_LOGS = 3000;

/// Shared, deeply-reactive application state (Svelte 5 runes).
export const appState = $state({
  os: null as OsInfo | null,
  config: null as Config | null,
  status: null as EnvStatus | null,
  logs: [] as LogLine[],
  progress: null as Stage | null,
  busy: false,
});

/// Detect installed components/versions. `checkLatest` also probes the network
/// for the newest uv release (slower); pass false for quick refreshes.
export async function refreshStatus(checkLatest = false): Promise<void> {
  if (!appState.config) return;
  try {
    appState.status = await api.detectStatus(appState.config, checkLatest);
  } catch (e) {
    pushLog({ level: "warn", message: `Could not detect status: ${e}` });
  }
}

export function pushLog(line: LogLine): void {
  appState.logs.push(line);
  if (appState.logs.length > MAX_LOGS) {
    appState.logs.splice(0, appState.logs.length - MAX_LOGS);
  }
}

export function clearLogs(): void {
  appState.logs = [];
  appState.progress = null;
}

export async function persistConfig(): Promise<void> {
  if (!appState.config) return;
  try {
    await api.saveConfig(appState.config);
  } catch (e) {
    pushLog({ level: "warn", message: `Could not save config: ${e}` });
  }
}

export async function setLanguage(l: string): Promise<void> {
  locale.set(l);
  if (appState.config) appState.config.language = l;
  await persistConfig();
}

/// Run an async action guarded by the busy flag, resetting the log panel first
/// and capturing any error into the log.
export async function runAction(
  fn: (cfg: Config) => Promise<void>,
): Promise<void> {
  if (!appState.config || appState.busy) return;
  clearLogs();
  appState.busy = true;
  try {
    await fn(appState.config);
  } catch (e) {
    pushLog({ level: "error", message: String(e) });
  } finally {
    appState.busy = false;
    // Reflect any changes an install made.
    void refreshStatus(false);
  }
}
