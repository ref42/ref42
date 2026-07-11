---
title: '一分钟，搭建Cortex-M+Rust开发环境'
description: '整理 Cortex-M 常用 Rust target 安装命令，覆盖 M0/M0+、M3、M4/M7、M4F/M7F 和 M33F。'
weight: 20
---

## 注意

这一部分非常简单，完成了[Rust开发环境](/toolchain/rust_dev_base_env/)搭建后，复制以下命令到命令行进行安装即可。

---

## 命令整理

以下给出一些安装命令，方便复制直接安装。

```bash
# M0/M0+内核芯片，常见芯片：STM32F0xx，PY32F0xx，....
rustup target add thumbv6m-none-eabi
```
```bash
# M3内核芯片，常见芯片：STM32F1xx，...
rustup target add thumbv7m-none-eabi
```
```bash
# M4和M7内核芯片，常见芯片：STM32F4xx，...
rustup target add thumbv7em-none-eabi
```
```bash
# M4F和M7F内核芯片，F意指带有浮点运算，常见芯片：STM32F4xx，STM32G4xx，STM32H7xx，...
rustup target add thumbv7em-none-eabihf
```
```bash
# M33F内核芯片，常见芯片：STM32U5xx，...
rustup target add thumbv8m.main-none-eabihf
```

---

## 总结

对于本博客会涉及到的所有芯片，其`target`都在上述`命令整理`里列举出来了，推荐直接全部安装。

---

## 常见问题

- 编译时报`can't find crate for core`时，通常是目标芯片对应的`target`没有安装。
- 不确定芯片属于哪个内核时，先查看芯片数据手册里的 CPU core 信息，再选择对应`target`。
- 带硬件浮点的芯片优先选择`eabihf`目标；如果工程或芯片配置不匹配，编译或运行时都可能异常。
