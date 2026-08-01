# pydev — Cross-Platform Python Environment Installer

## Overview

pydev is a graphical tool that sets up a complete Python development
environment on **Windows, Linux, and macOS**. The guiding principle is that a
single click should install everything needed with sensible defaults, while
still allowing customization for users who want it.

## Target Users

The primary audience is **beginners** who may not be familiar with common
developer tools. Defaults, wording, and flows should assume little prior
knowledge and minimize the number of decisions a user has to make.

## Localization

- The UI supports both **Chinese** and **English**.
- **Chinese is the default**; the user can switch to English at any time.
- The design should make it easy to add other popular languages later.

## Components Installed

1. **uv** — the Python package and environment manager.
2. **Python** (installed via uv)
   - Defaults to the current most stable release.
   - The user can select other available versions.
   - The tool can be re-launched to install a **single component only** — for
     example, adding another Python version without reinstalling the rest.
3. **VSCode** — the latest stable release, together with the extensions needed
   for Python development.

## PATH & Environment

The tool must update every relevant `PATH` entry so the installed tools work
without further setup:

- **Windows** — the user/system environment variables.
- **Linux / macOS** — the shell profile, defaulting to `~/.bashrc`, with the
  option to target other popular shells.

## One-Click Installation

Ideally the user only needs a single click: the tool then installs all required
components using reasonable default configuration.

## Network Test & Proxy

- Provide a **"network test"** action so users can check their current
  connectivity.
- Proxy settings default to empty but can be configured, for users who are
  behind a firewall.

## Command-Line Interface

- In addition to the GUI, the tool provides a **CLI** that achieves the same
  result.
- It ships with a default config file (or a config example) to drive CLI runs.
- For the CLI, installing VSCode is **optional**, since some users only have a
  terminal environment.

## Packaging

The project must ship both:

- a standalone **executable**, and
- an **installer package**.

## Technical Constraints

Choose the implementation language and toolchain that produces the **smallest
possible executable and installer footprint**.
