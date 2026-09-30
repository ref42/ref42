---
title: 'Configure an ESP32 template project'
description: 'Use esp-generate to create an ESP32-C3 Rust starter project and configure espflash, logging, debugging, and VSCode integration.'
weight: 10
date: "2026-07-11"
tags: [esp32, toolchain]
tested_with: ["Rust 1.89.0", "espflash 4.0.1", "probe-rs 0.29.1"]
---

## Create a starter project with esp-generate

> Before starting, complete the [base Rust environment setup](/toolchain/rust_dev_base_env/), then finish the [ESP32 Rust setup](/toolchain/esp32-dev-setup/).

After completing those steps, create or open a project directory and run the following command in a terminal. You can also run `esp-generate` by itself and follow its prompts.

```bash
esp-generate --chip=esp32c3 esp32c3_embassy_dht11_demo
```
The command is made up of these parts:

```bash
esp-generate # invoke the generator
--chip=esp32c3  # select the chip
esp32c3_embassy_dht11_demo  # project name; snake_case is recommended
```

The terminal opens an interactive, GUI-like menu. Use the arrow keys to highlight a `feature`, then press `Enter` to select it; selected features show a ✅ marker.

![esp-generate feature selection screen](/esp32/template-project/esp-generate-00.png)

A `feature` prefixed with [▶] contains child features. Highlight it with the arrows and press `Enter` to open its submenu.

![esp-generate feature group screen](/esp32/template-project/esp-generate-01.png)

Highlight `▶  Flashing, logging and debugging (espflash)` and press `Enter` to open its submenu.

![espflash feature group in esp-generate](/esp32/template-project/esp-generate-02.png)

Select the features shown in the next screen; in this example, options `2` and `3` are selected.

![selected flashing and logging features in esp-generate](/esp32/template-project/esp-generate-03.png)

Note: options `1` and `2` are mutually exclusive. Selecting both will cause a project error.

Press `ESC` to return to the main menu and continue enabling features.

Highlight `▶  Optional editor integration` and press `Enter` to open its submenu.

![optional editor integration menu in esp-generate](/esp32/template-project/esp-generate-04.png)

This menu selects your editor. The generator can configure the project for it and recommend useful extensions.

![VSCode editor integration selected in esp-generate](/esp32/template-project/esp-generate-05.png)

After selecting the editor feature, press `ESC` to return to the main menu.

All features are now selected. Press `S` for `Save` to generate the project; the terminal will show its progress.

![esp-generate project creation complete](/esp32/template-project/esp-generate-06.png)

```bash
🆗 Rust (stable): 1.89.0
🆗 espflash: 4.0.1
🆗 probe-rs: 0.29.1
```

The template is ready. Future projects follow the same process, and tool versions may differ as the toolchain evolves.
