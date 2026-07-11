---
title: '一分钟，搭建ESP32 Rust开发环境'
description: '安装 esp-generate、espup 和 espflash，完成 ESP32 Rust 项目的生成、工具链安装和烧录准备。'
weight: 30
---

## 注意

与`Cortex-M`系列单片机开发不同，`ESP32`的工具链稍显特殊，但是都属于一键式安装，遵循以下命令安装即可。

---

## 按照顺序，执行以下命令

```bash
# 根据模板生成工程
cargo install esp-generate
# esp32开发包安装工具
cargo binstall espup
# 安装esp32开发包
espup install
# 安装烧录工具
cargo install espflash
```

完成以上步骤之后，`ESP32`的开发环境就搭建完毕了。恭喜，你的技术栈又丰富了许多。

## 检查安装结果

```bash
esp-generate --version
espup --version
espflash --version
```

如果以上命令都能正常输出版本号，就可以继续阅读[ESP32模板工程](/esp32/esp-generate-template/)。

## 常见问题

- `cargo binstall espup`失败时，可以先安装`cargo-binstall`，或者改用`cargo install espup`。
- `espup install`完成后如果终端找不到 ESP 相关环境变量，重启终端后再试。
- 烧录失败时，检查 USB 串口驱动、开发板端口占用和数据线是否支持传输。
