export type Os = "windows" | "macos" | "linux";

export interface OsInfo {
  os: Os;
  arch: string;
}

export type LogLevel = "info" | "warn" | "error" | "success" | "cmd";

export interface LogLine {
  level: LogLevel;
  message: string;
}

export interface Stage {
  key: string;
  index: number;
  total: number;
}

export interface EndpointResult {
  name: string;
  url: string;
  ok: boolean;
  status: number | null;
  latency_ms: number | null;
  error: string | null;
}

export interface PathPreview {
  entries: string[];
  targets: string[];
}

export interface ProxyConfig {
  http: string;
  https: string;
  no_proxy: string;
}

export interface Config {
  language: string;
  reinstall_existing: boolean;
  proxy: ProxyConfig;
  uv: { version: string };
  python: { version: string; set_default: boolean };
  vscode: { install: boolean; extensions: string[] };
  path: { update: boolean; shells: string[] };
}

export interface Tool {
  installed: boolean;
  current: string | null;
  latest: string | null;
  upgrade_available: boolean;
  location: string | null;
}

export interface ExtStatus {
  id: string;
  installed: boolean;
  version: string | null;
  latest: string | null;
  upgrade_available: boolean;
}

export interface PythonStatus {
  installed_versions: string[];
  requested: string;
  satisfied: boolean;
}

export interface PathStatus {
  configured: boolean;
  pending_targets: string[];
}

export interface EnvStatus {
  uv: Tool;
  python: PythonStatus;
  vscode: Tool;
  extensions: ExtStatus[];
  path: PathStatus;
}
