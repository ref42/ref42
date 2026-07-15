---
title: 'npnp：开源电子元件库导出工具'
description: 'npnp 是一个用 Rust 编写的开源 EDA 元件库导出工具，可以生成 Altium 和 KiCad 可用的符号、封装和 3D 模型库。'
weight: 10
---

<p align="center">
  <img src="/utils/npnp.png" alt="npnp logo" width="260">
</p>

## 简介

`npnp` 是一个面向电子工程工作流的开源命令行工具，全称可以理解为 `Normalize Pin Net Pad`。它用 `Rust` 编写，用来把线上元件数据整理成可以检查和使用的 EDA 库。

> 仓库地址：https://github.com/ref42/npnp

项目已经重新开源。欢迎通过 `Issue` 反馈问题、提供复现样例或提出需求；目前不合并外部 `Pull Request`。

## 解决什么问题

画板时经常会遇到同一个问题：元件本身可以找到，但符号、封装、3D 模型和元数据分散在不同格式里，手动整理很慢，也容易出错。

`npnp` 把这些步骤收敛成一个可重复的导出流程：

- 搜索或读取 LCSC 元件 ID。
- 下载上游符号、封装和 3D 模型数据。
- 生成 Altium `.SchLib` 和 `.PcbLib`。
- 生成 KiCad `.kicad_sym`、`.pretty` 和 `.3dshapes`。
- 支持批量导出、合并库、追加到已有合并库。
- 支持中英文元数据，缺失时尽量回退到已有来源数据。

## 当前支持

| 目标 | 输出 |
| --- | --- |
| Altium | `.SchLib`、`.PcbLib` |
| KiCad | `.kicad_sym`、`.pretty`、`.3dshapes` |
| 3D 模型 | STEP 或 OBJ/MTL，取决于上游数据 |
| 批量流程 | 文本文件输入、并行导出、失败继续、合并与追加 |

## 快速使用

可以从 GitHub Releases 下载对应平台的二进制文件：

> https://github.com/ref42/npnp/releases

常用命令示例：

```bash
npnp search C2040 --limit 5

npnp altium export C2040 --full --output altium-libs --force
npnp altium batch --input ids.txt --output generated/altium --merge --library-name MyLib --full --continue-on-error

npnp kicad export C2040 --full --output kicad-libs --library-name MyParts --force
npnp kicad batch --input ids.txt --output generated/kicad --library-name MyParts --full --force --parallel 4 --continue-on-error
```

如果不确定该怎么组合参数，可以运行：

```bash
npnp --prompt
```

## 注意事项

自动生成库不是跳过检查的理由。`npnp` 的目标是减少重复劳动，而不是替代工程判断。导出的符号、封装和 3D 模型在进入正式项目之前，仍然建议在 EDA 工具里做一次目视检查。

后续会继续改进一些边界情况，例如非常规焊盘的阻焊层处理，以及部分 3D 模型里的 logo/watermark 几何体清理。
