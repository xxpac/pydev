import { invoke } from "@tauri-apps/api/core";
import type {
  Config,
  EndpointResult,
  EnvStatus,
  OsInfo,
  PathPreview,
} from "./types";

/// Thin typed wrappers around the Rust `#[tauri::command]`s.
export const api = {
  detectPlatform: () => invoke<OsInfo>("detect_platform"),
  getConfig: () => invoke<Config>("get_config"),
  saveConfig: (config: Config) => invoke<void>("save_config", { config }),
  listPythonVersions: (config: Config) =>
    invoke<string[]>("list_python_versions", { config }),
  detectStatus: (config: Config, checkLatest: boolean) =>
    invoke<EnvStatus>("detect_status", { config, checkLatest }),
  pathPreview: (config: Config) => invoke<PathPreview>("path_preview", { config }),
  testNetwork: (config: Config) =>
    invoke<EndpointResult[]>("test_network", { config }),
  oneClick: (config: Config) => invoke<void>("one_click", { config }),
  installUv: (config: Config) => invoke<void>("install_uv", { config }),
  installPython: (config: Config) => invoke<void>("install_python", { config }),
  installVscode: (config: Config) => invoke<void>("install_vscode", { config }),
  installExtensions: (config: Config) =>
    invoke<void>("install_extensions", { config }),
  applyPath: (config: Config) => invoke<void>("apply_path", { config }),
};
