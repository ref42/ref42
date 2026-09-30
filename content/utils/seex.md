---
title: 'SeEx: desktop EDA library exporter'
description: 'SeEx is a Rust desktop application built with egui/eframe for batch-exporting LCSC parts to KiCad and Altium Designer.'
weight: 20
date: "2026-07-15"
updated: "2026-08-08"
tags: [tools, eda, kicad]
---

<p align="center">
  <img src="/utils/seex.svg" alt="SeEx logo" width="220">
</p>

`SeEx` (Seek and Export) combines part search, list management, and EDA library export in one desktop workflow. It supports KiCad and Altium Designer and can process many parts at once.

> [Repository](https://github.com/ref42/seex) · [Releases](https://github.com/ref42/seex/releases)

## Features

- Export KiCad symbols, footprints, and 3D models.
- Export Altium Designer `SchLib` and `PcbLib` files.
- Export one part, many parts, or merge exports into one library.
- Support bilingual metadata, multithreaded downloads, and standalone 3D model export.
- Customize schematic outline and fill colors.
- Known support includes KiCad 9.0+ and Altium Designer 23.x+.

## Usage

After opening SeEx, review part inputs on the Monitor page, then use Export to choose the target EDA tool, metadata language, export scope, and batch options. It is useful when you repeatedly export libraries without wanting to memorize commands.

SeEx includes the exporter directly and no longer requires a separate `npnp` CLI installation. For scripts, CI, or a command-line-only workflow, use [`npnp`](/utils/npnp/).

## Interface preview

<div class="image-pair">
  <img src="/utils/seex/1.png" alt="SeEx interface preview 1">
  <img src="/utils/seex/2.png" alt="SeEx interface preview 2">
</div>

<div class="image-pair">
  <img src="/utils/seex/3.png" alt="SeEx interface preview 3">
  <img src="/utils/seex/4.png" alt="SeEx interface preview 4">
</div>

## Technology

SeEx is built with Rust, egui, and eframe to make the path from finding parts to organizing a list and exporting a library more direct.
