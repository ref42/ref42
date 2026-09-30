---
title: 'ARMUP user guide'
description: 'Install and update the Cortex-M embedded toolchain on Windows with ARMUP.'
weight: 30
date: "2026-07-11"
tags: [tools, cortex-m, windows]
---

## Overview

`ARMUP` is a `Rust` toolchain installer for `Cortex-M` development on `Windows`. It downloads common tools into one root directory and can add them to the current user's `Path`.

> Repository: https://github.com/ref42/armup

If you want a working `Cortex-M` environment on `Windows` without manually downloading, extracting, and configuring many tools, `ARMUP` is designed for that job.

## What it installs

The current tool list is:

| Tool | Purpose |
| --- | --- |
| `arm-none-eabi-gcc` | Arm GNU Toolchain for traditional Cortex-M cross-compilation and debugging |
| `clangd` | C/C++ language server for editor completion and navigation |
| `cmake` | Build-system generator |
| `ninja` | Fast build runner |
| `probe-rs` | Rust embedded flashing and debugging tool |
| `xpack-openocd` | OpenOCD debugging and flashing service |

The default installation root is:

```
D:\Embedded_Toolchain
```

Use `--root` to choose another location. A short path without spaces is easier to troubleshoot.

## Quick install

The simplest installation command is:

```
armup install -a --root D:\Embedded_Toolchain -j 24
```

The options mean:

- `install`: perform an installation.
- `-a`: install every supported tool.
- `--root D:\Embedded_Toolchain`: use this installation root.
- Installed tool paths are added to the current user's `Path` by default.
- `-j 24`: use more parallel downloads when the network allows it.

After installation, open a new terminal so the updated `Path` takes effect.

## Update tools

To update installed tools later, run:

```
armup update -a --root D:\Embedded_Toolchain -j 24
```

`ARMUP` checks the tools in the root, resolves the newest supported versions, and updates only what is needed. It then removes old version directories and refreshes the user's `Path`.

To update only selected tools:

```
armup update --tool probe-rs,xpack-openocd --root D:\Embedded_Toolchain
```

## Check status

After installation, inspect the status:

```
armup status --root D:\Embedded_Toolchain
```

Add `--verbose` to show executable paths and `Path` entries:

```
armup status --root D:\Embedded_Toolchain --verbose
```

The output lists installed versions and whether the user's `Path` needs an update.

## Run diagnostics

If installation or updating fails, run:

```
armup doctor --root D:\Embedded_Toolchain
```

`doctor` checks the installation root, temporary staging directory, network requests, version resolution, and user `Path` registry access.

## Select specific tools

To install only selected tools:

```
armup install --tool probe-rs,ninja --root D:\Embedded_Toolchain
```

Supported tool names are:

```
arm-none-eabi-gcc
clangd
cmake
ninja
probe-rs
xpack-openocd
```

## Select versions

By default, `ARMUP` installs the newest supported version. To choose from recent upstream versions interactively:

```
armup install -a --root D:\Embedded_Toolchain --select-versions
```

Note: `--select-versions` requires an interactive terminal. Do not use it in scripts, CI, or other non-interactive environments.

## Verify after installation

Open a new terminal and check each version:

```
arm-none-eabi-gcc --version
clangd --version
cmake --version
ninja --version
probe-rs --version
openocd --version
```

If every command prints a version, the toolchain and `Path` are ready. You can install the chip's `Rust target` and use `probe-rs` or `OpenOCD` for flashing and debugging.

## Troubleshooting

- Non-interactive mode still requires `--root` because the installation directory must be explicit.
- Existing terminals do not refresh after a `Path` change; open a new terminal to test.
- If a download fails, run `armup doctor` to check the network, proxy, and upstream release resolution.
- To leave the user's `Path` unchanged, replace `--add-path` with `--no-add-path`.
- If old versions are present, an update keeps the newest required versions and removes unused directories.
- Avoid deeply nested system paths and paths with spaces; scripts and tool invocations are less reliable there.

## Closing notes

`ARMUP` addresses a practical problem: setting up an embedded toolchain on `Windows` is often difficult because of download sources, archive layouts, environment variables, and ongoing updates. A single command makes new projects much faster to start.

Bug reports and feedback are welcome. Enjoy 😊.
