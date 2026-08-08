---
title: 'SeEx：EDA 元件库导出的桌面工具'
description: 'SeEx 是一个用 Rust 和 egui/eframe 构建的桌面工具，用于批量导出 LCSC 元件到 KiCad 和 Altium Designer。'
weight: 20
---

<p align="center">
  <img src="/utils/seex.png" alt="SeEx logo" width="220">
</p>

`SeEx`（Seek and Export）把元件搜索、清单整理和 EDA 库导出集中到一个桌面工作流里。它支持 KiCad 与 Altium Designer，并且可以批量处理多个元件。

> [项目仓库](https://github.com/ref42/seex) · [下载 Releases](https://github.com/ref42/seex/releases)

## 主要能力

- 导出 KiCad 的 Symbol、Footprint 和 3D 模型。
- 导出 Altium Designer 的 `SchLib` 和 `PcbLib`。
- 支持单个导出、批量导出，以及合并到同一个库。
- 支持中英文元数据、多线程下载和独立导出 3D 模型。
- 支持自定义原理图描边与填充颜色。
- 当前已知支持 KiCad 9.0+ 与 Altium Designer 23.x+。

## 使用方式

打开 SeEx 后，可以在 Monitor 页面确认元件输入，再到 Export 页面选择目标 EDA、元数据语言、导出范围和批量选项。适合不想频繁记忆命令，又需要重复导出元件库的场景。

SeEx 当前已经直接包含导出能力，不再依赖单独安装的 `npnp` CLI。需要脚本、CI 或纯命令行工作流时，可以使用同系列的 [`npnp`](/utils/npnp/)。

## 界面预览

<div class="image-pair">
  <img src="/utils/seex/1.png" alt="SeEx interface preview 1">
  <img src="/utils/seex/2.png" alt="SeEx interface preview 2">
</div>

<div class="image-pair">
  <img src="/utils/seex/3.png" alt="SeEx interface preview 3">
  <img src="/utils/seex/4.png" alt="SeEx interface preview 4">
</div>

## 技术栈

SeEx 使用 Rust、egui 和 eframe 构建，目标是让“找到元件、整理清单、导出库”这条路径更直接。
