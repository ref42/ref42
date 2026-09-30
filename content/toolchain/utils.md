---
title: "Install useful tools"
description: 'Install probe-rs-tools, cargo-binutils, and llvm-tools for flashing, resetting, and analyzing Rust embedded firmware.'
weight: 40
date: "2026-07-11"
tags: [toolchain, probe-rs]
---

## Flashing and debugging tools
With `cargo`, probe-rs can download and run a program in one step. It can also be used directly to flash and reset a chip.

```bash
cargo install probe-rs-tools
```
### Use probe-rs directly

```bash
# Flash an xxx.elf firmware image to the target chip
probe-rs download xxx.elf --chip STM32F405RG
# Reset the target chip
probe-rs reset --chip STM32F405RG
```
## Supporting tools

```bash
cargo install cargo-binutils
rustup component add llvm-tools
```
These commands provide the following tools:

| Cargo wrapper | Native Rust tool |
| -------------- | ----------------- |
| `cargo-cov.exe` | `rust-ar.exe`     |
| `cargo-nm.exe`  | `rust-as.exe`     |
| `cargo-objcopy.exe` | `rust-cov.exe` |
| `cargo-objdump.exe` | `rust-ld.exe`  |
| `cargo-profdata.exe` | `rust-lld.exe` |
| `cargo-readobj.exe` | `rust-nm.exe`  |
| `cargo-size.exe`   | `rust-objcopy.exe` |
| `cargo-strip.exe`  | `rust-objdump.exe` |
|                    | `rust-profdata.exe` |
|                    | `rust-readobj.exe`  |
|                    | `rust-size.exe`     |
|                    | `rust-strip.exe`    |

Use `cargo-size` to inspect firmware size.

```bash
D:\MCU-Projects\STM32-Projects\RUST\h7\embassy\embassy_h7_blink>cargo size
    Finished `dev` profile [optimized + debuginfo] target(s) in 0.16s
   text    data     bss     dec     hex filename
  31488      80    5532   37100    90ec embassy_h7_blink
```

## Troubleshooting

- If `probe-rs` cannot find a chip, check that `--chip` matches its supported list.
- If flashing fails, check the debugger connection, board power, and debug state.
- If `cargo size` does not run, install `cargo-binutils` and add the `llvm-tools` component.
