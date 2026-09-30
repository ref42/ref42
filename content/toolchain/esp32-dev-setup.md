---
title: 'Set up an ESP32 Rust environment in one minute'
description: 'Install esp-generate, espup, and espflash to create ESP32 Rust projects and prepare the flashing toolchain.'
weight: 30
date: "2026-07-11"
tags: [esp32, toolchain]
---

## Before you start

Unlike the `Cortex-M` workflow, the `ESP32` toolchain has a few special steps, but each is a one-command installation.

---

## Run these commands in order

```bash
# Generate a project from a template
cargo install esp-generate
# ESP32 toolchain installer
cargo binstall espup
# Install the ESP32 toolchain
espup install
# Install the flashing tool
cargo install espflash
```

After these commands, the `ESP32` development environment is ready.

## Verify the installation

```bash
esp-generate --version
espup --version
espflash --version
```

If every command prints a version, continue with the [ESP32 template project](/esp32/esp-generate-template/).

## Troubleshooting

- If `cargo binstall espup` fails, install `cargo-binstall` first or use `cargo install espup`.
- If the terminal cannot find ESP variables after `espup install`, restart the terminal.
- If flashing fails, check the USB serial driver, port usage, and whether the cable supports data transfer.
