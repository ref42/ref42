---
title: 'SeEx：npnp 的桌面图形界面'
description: 'SeEx 是 npnp 的开源桌面 GUI，可以监听剪贴板里的 LCSC 元件 ID，维护 BOM 列表，并导出 Altium 或 KiCad 元件库。'
weight: 20
---

<p align="center">
  <img src="/utils/seex.png" alt="SeEx logo" width="220">
</p>

## 简介

`SeEx` 是 `Seek & Export` 的缩写，它是 [`npnp`](/utils/npnp/) 的桌面图形界面。它会监听剪贴板内容，按关键字或正则提取 LCSC 元件 ID，然后把这些 ID 交给 `npnp` 导出 EDA 库。

> 仓库地址：https://github.com/ref42/seex

项目已经重新开源。欢迎通过 `Issue` 反馈问题、提供复现样例或提出需求；目前不合并外部 `Pull Request`。

## 适合什么场景

如果你在 LCSC 或类似页面上选型，经常复制一串元件信息，那么 `SeEx` 可以把这个动作变成一个可追踪的清单：

- 监听剪贴板并识别 `Cxxxx` 元件 ID。
- 同一段文本里出现多个 ID 时，尽量全部提取。
- 维护当前匹配列表和历史记录。
- 导出 BOM CSV，方便后续整理。
- 调用 `npnp` 导出 Altium 或 KiCad 库。
- 导出失败时保留详细信息，方便复制失败 ID 排查。

## 和 npnp 的关系

`npnp` 是底层导出引擎，适合命令行和脚本；`SeEx` 是桌面 GUI，适合选型、复制、整理、批量导出的交互流程。

| 工具 | 角色 |
| --- | --- |
| `npnp` | 负责搜索、下载、转换和导出 EDA 库 |
| `SeEx` | 负责监听剪贴板、管理待导出 ID、提供图形化导出入口 |

## 下载和使用

可以从 GitHub Releases 下载对应平台的安装包：

> https://github.com/ref42/seex/releases

打开 `SeEx` 后，一边在网页上复制元件信息，一边在 Monitor 页面确认识别结果。整理好 ID 后，可以在 Export 页面选择：

- EDA 目标：`Altium` 或 `KiCad`
- 元数据语言：中文或英文
- 导出范围：完整库、符号库、封装库等
- 批量选项：合并、追加、失败继续、强制覆盖等

## 技术栈

`SeEx` 使用 `Tauri` 构建桌面应用，前端使用 `TypeScript`，底层导出能力来自 `Rust` 版本的 `npnp`。

这类工具的目标不是做一个复杂的管理系统，而是把“复制元件 ID → 保存清单 → 导出库”这个高频流程做得足够顺手。
