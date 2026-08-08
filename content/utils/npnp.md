---
title: 'npnp：EDA 元件库导出 CLI'
description: 'npnp 是一个用 Rust 编写的命令行工具，可以把 LCSC 元件数据导出为 Altium Designer 和 KiCad 可用的库。'
weight: 10
---

<p align="center">
  <img src="/utils/npnp.png" alt="npnp logo" width="260">
</p>

`npnp`（Normalize Pin Net Pad）面向脚本、批处理和 CI 工作流，把 LCSC / EasyEDA 元件数据导出为可以检查和使用的 EDA 库。

> [项目仓库](https://github.com/ref42/npnp) · [下载 Releases](https://github.com/ref42/npnp/releases)

## 主要能力

- 导出 KiCad 的 Symbol、Footprint 和 3D 模型。
- 导出 Altium Designer 的 `SchLib` 和 `PcbLib`。
- 支持单个元件、文本文件批量导出，以及并行处理。
- 支持合并库、向已有合并库追加元件和独立导出 3D 模型。
- 支持中英文元数据，并在目标语言缺失时回退到源数据。
- 支持 Windows、macOS 和 Linux 发布包。
- 当前已知支持 KiCad 9.0+ 与 Altium Designer 23.x+。

## 常用命令

```bash
npnp search C2040 --limit 5

npnp altium export C2040 --full --output altium-libs --force
npnp altium batch --input ids.txt --output generated/altium --merge --library-name MyLib --full --continue-on-error

npnp kicad export C2040 --full --output kicad-libs --library-name MyParts --force
npnp kicad batch --input ids.txt --output generated/kicad --library-name MyParts --full --force --parallel 4 --continue-on-error
```

不确定参数组合时，可以运行 `npnp --prompt` 查看可直接复制的命令。

## 导出预览

<div class="image-pair">
  <img src="/utils/npnp/kicad_symbol.png" alt="KiCad symbol export">
  <img src="/utils/npnp/kicad_footprint.png" alt="KiCad footprint export">
</div>

<div class="image-pair">
  <img src="/utils/npnp/kicad_3dmodel.png" alt="KiCad 3D model export">
  <img src="/utils/npnp/pcb_01.png" alt="Altium PCB library export">
</div>

<div class="image-pair">
  <img src="/utils/npnp/pcb_02.png" alt="Altium PCB library export detail">
  <img src="/utils/npnp/sch_01.png" alt="Altium schematic library export">
</div>

## 使用提醒

自动生成库并不替代工程检查。将库用于正式项目之前，仍然建议在目标 EDA 工具中检查符号、封装、3D 模型和元数据。

需要图形界面时，可以使用同系列的 [`SeEx`](/utils/seex/)；需要命令行、脚本或 CI 时，`npnp` 更合适。
