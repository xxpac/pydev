# pydev

A tiny, cross-platform GUI **and** CLI that sets up a complete Python development
environment for beginners in one click: the [uv](https://docs.astral.sh/uv/)
package manager, a Python interpreter (installed via uv), and
[Visual Studio Code](https://code.visualstudio.com/) with the right extensions -
including all the `PATH` changes.

一个跨平台的图形界面 / 命令行小工具：一键为初学者安装 uv、Python 和 VSCode 及其
Python 扩展，并自动配置 `PATH`。界面默认中文，可切换英文。

Built with **Rust + Tauri v2** (system WebView, so installers are only a few MB)
and a **Svelte** front-end. A shared Rust `core` crate powers both the GUI and
the CLI.

## Features

- One-click install of uv + Python + VSCode + extensions with sensible defaults.
- Install components individually and re-run anytime (e.g. add another Python
  version).
- uv installs and manages Python (`uv python install`), so no separate Python
  download step.
- Handles `PATH`: user registry on Windows; `~/.bashrc` (or `~/.zshrc`,
  fish, `~/.profile`) on Unix - no administrator rights required.
- Network test with an optional proxy for users behind a firewall.
- Bilingual UI (Chinese default / English), easy to extend with more locales.
- CLI with a config file for terminal-only setups (VSCode optional).

## Download

Grab the installer for your OS from the
[Releases](../../releases) page (built by CI):

- **Windows:** `.msi` or `.exe` (NSIS). The WebView2 runtime is fetched
  automatically if missing.
- **macOS:** `.dmg` (universal).
- **Linux:** `.deb`, `.rpm`, or `.AppImage`.

Prefer the terminal? Download the standalone `pydev-cli` binary from the same
release.

## Using the GUI

1. Launch **pydev**.
2. (Optional) Open **Network** and click *Test network*; if you are behind a
   firewall, set an HTTP/HTTPS proxy.
3. On **One-click**, review what will be installed and press the button.
4. Watch progress stream in the activity log at the bottom.

Or use the **Components** tab to install a single piece, the **PATH** tab to
review/apply environment changes, and **Settings** to switch language.

## Using the CLI

```bash
# Generate a config file you can edit
pydev-cli init --output config.toml

# Install everything
pydev-cli install --config config.toml

# Install just one component
pydev-cli install --config config.toml --only python
pydev-cli install --only uv          # uses built-in defaults when no --config

# Other helpers
pydev-cli test-network --config config.toml
pydev-cli list-python
pydev-cli path-preview --config config.toml
```

See [`config.example.toml`](config.example.toml) for every option. Terminal-only
users can set `[vscode] install = false` to skip the editor.

## Build from source

### Prerequisites

- [Rust](https://rustup.rs/) (stable) and [Node.js](https://nodejs.org/) LTS.
- **Linux only** - Tauri system libraries:
  ```bash
  sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev \
      librsvg2-dev patchelf xdg-utils file
  ```
- The Tauri CLI comes from `npm` (installed by `npm install`).

### Develop

```bash
npm install
npm run tauri dev      # hot-reloading GUI
```

### Release builds

```bash
# GUI app + native installers for the current OS
npm run tauri build

# Just one bundle type, e.g. a Debian package
npm run tauri build -- --bundles deb

# The standalone CLI
cargo build --release -p pydev-cli   # -> target/release/pydev-cli
```

Cross-OS installers are produced by the
[release workflow](.github/workflows/release.yml): push a tag like `v0.1.0` and
GitHub Actions builds Windows/macOS/Linux installers plus the CLI and attaches
them to a draft release.

> Note: Tauri builds are per-OS. From a Linux/WSL box you can produce the Linux
> artifacts locally; Windows and macOS installers come from CI.

## Project layout

```
pydev/
├─ crates/
│  ├─ core/         # shared engine: runner, downloader, installers, PATH, netcheck
│  └─ cli/          # pydev-cli (clap), reads config.toml
├─ src-tauri/       # Tauri v2 app: #[tauri::command]s bridge the UI to core
├─ src/             # Svelte front-end (screens + i18n)
├─ config.example.toml
└─ .github/workflows/release.yml
```

The GUI never runs raw shell from the WebView: every privileged action is a Rust
command in `src-tauri` that calls into `core` and streams `install://log` /
`install://progress` events back to the UI.

## How installs work

- **uv** - official standalone installer (`install.sh` / `install.ps1`) with
  `UV_INSTALL_DIR` and `UV_NO_MODIFY_PATH=1` so pydev owns PATH changes.
- **Python** - `uv python install [version] --default`.
- **VSCode** - latest stable per-OS download: silent Inno-Setup install on
  Windows, `ditto`-extracted app on macOS, tarball + symlink on Linux; then
  `code --install-extension` for each extension.
- **Proxy** - applied to downloads and to child installers via `HTTP(S)_PROXY`.

## License

MIT
