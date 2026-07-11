---
title: "实用工具安装"
description: '安装 probe-rs-tools、cargo-binutils 和 llvm-tools，用于 Rust 嵌入式固件烧录、复位和体积分析。'
weight: 40
---

## 下载/调试工具
配合`cargo`可以一键下载运行程序，也可以单独使用，用于烧录并复位芯片。

```bash
cargo install probe-rs-tools
```
### 单独使用 probe-rs

```bash
# 将xxx.elf固件烧录到目标芯片
probe-rs download xxx.elf --chip STM32F405RG
# 复位目标芯片
probe-rs reset --chip STM32F405RG
```
## 零碎工具

```bash
cargo install cargo-binutils
rustup component add llvm-tools
```
包含以下这些工具

| Cargo 封装工具 | Rust 原生底层工具 |
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

使用`cargo-size `查看固件大小

```bash
D:\MCU-Projects\STM32-Projects\RUST\h7\embassy\embassy_h7_blink>cargo size
    Finished `dev` profile [optimized + debuginfo] target(s) in 0.16s
   text    data     bss     dec     hex filename
  31488      80    5532   37100    90ec embassy_h7_blink
```

## 常见问题

- `probe-rs`找不到芯片时，确认`--chip`名称是否与 probe-rs 支持列表一致。
- 烧录失败时，先检查调试器连接、目标板供电和芯片是否处于可调试状态。
- `cargo size`无法运行时，确认已经安装`cargo-binutils`并添加`llvm-tools`组件。
