---
title: 'Set up a Cortex-M + Rust environment in one minute'
description: 'Common Rust target installation commands for Cortex-M cores: M0/M0+, M3, M4/M7, M4F/M7F, and M33F.'
weight: 20
date: "2026-07-11"
tags: [toolchain, cortex-m]
---

## Before you start

This step is simple. After completing the [Rust environment setup](/toolchain/rust_dev_base_env/), copy the commands below into a terminal.

---

## Commands

Use the commands below to install the targets you need.

```bash
# M0/M0+ core; common chips: STM32F0xx, PY32F0xx, ...
rustup target add thumbv6m-none-eabi
```
```bash
# M3 core; common chips: STM32F1xx, ...
rustup target add thumbv7m-none-eabi
```
```bash
# M4 and M7 cores; common chips: STM32F4xx, ...
rustup target add thumbv7em-none-eabi
```
```bash
# M4F and M7F cores; F means hardware floating point. Common chips: STM32F4xx, STM32G4xx, STM32H7xx, ...
rustup target add thumbv7em-none-eabihf
```
```bash
# M33F core; common chips: STM32U5xx, ...
rustup target add thumbv8m.main-none-eabihf
```

---

## Summary

Every chip covered by this site uses one of the `target` values listed above. Installing all of them is convenient.

---

## Troubleshooting

- If compilation reports `can't find crate for core`, the chip's `target` is usually missing.
- If you are unsure which core a chip uses, check its datasheet and select the matching `target`.
- Prefer an `eabihf` target for hardware floating-point chips. A mismatch between the project and chip can fail at build or run time.
