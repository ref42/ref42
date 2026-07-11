---
title: 'ARMUP 使用指南'
description: '使用 ARMUP 在 Windows 上一键安装和更新 Cortex-M 嵌入式开发工具链。'
weight: 30
---

## 简介

`ARMUP`是一个面向`Windows`的`Cortex-M`嵌入式工具链安装器，用`Rust`编写。它会把常用工具下载并安装到同一个根目录下，也可以顺手把这些工具加入当前用户的`Path`。

> 仓库地址：https://github.com/ref42/armup

如果你只是想在`Windows`上快速准备一套可用的`Cortex-M`开发环境，而不想手动下载、解压、配置一堆工具，那么`ARMUP`就是为这个场景准备的。

## 会安装什么

目前支持以下工具：

| 工具 | 用途 |
| --- | --- |
| `arm-none-eabi-gcc` | Arm GNU Toolchain，用于传统 Cortex-M 交叉编译和调试辅助 |
| `clangd` | C/C++ language server，方便编辑器补全和跳转 |
| `cmake` | 常见构建系统生成器 |
| `ninja` | 快速构建执行器 |
| `probe-rs` | Rust 嵌入式烧录和调试工具 |
| `xpack-openocd` | OpenOCD 调试和烧录服务 |

默认安装根目录是：

```
D:\Embedded_Toolchain
```

你也可以通过`--root`指定到其他位置。建议放在一个路径简单、没有中文和空格的目录下，后续排查问题会省很多事。

## 喂饭

最直接的安装命令：

```
armup install -a --root D:\Embedded_Toolchain -j 24
```

这条命令的意思是：

- `install`：执行安装。
- `-a`：安装所有支持的工具。
- `--root D:\Embedded_Toolchain`：把工具安装到这个根目录。
- 默认会把安装后的工具路径加入当前用户`Path`。
- `-j 24`：提高并行下载连接数，网络条件允许时会更快。

安装完成后，重新打开一个终端，让新的`Path`生效。

## 更新工具

后续想更新已安装工具，可以运行：

```
armup update -a --root D:\Embedded_Toolchain -j 24
```

`ARMUP`会检查当前根目录下已经安装的工具，解析最新支持版本，只更新确实需要更新的部分。更新完成后，也会清理旧版本目录，并刷新当前用户`Path`。

如果只想更新某几个工具，可以这样写：

```
armup update --tool probe-rs,xpack-openocd --root D:\Embedded_Toolchain
```

## 查看状态

安装完成后，可以先看一下状态：

```
armup status --root D:\Embedded_Toolchain
```

如果需要查看可执行文件路径和`Path`条目，可以加`--verbose`：

```
armup status --root D:\Embedded_Toolchain --verbose
```

它会列出已经安装的工具版本，并提示当前用户`Path`是否需要更新。

## 自检

如果安装或更新不顺利，可以先跑：

```
armup doctor --root D:\Embedded_Toolchain
```

`doctor`会检查安装根目录、临时 staging 目录、网络请求、版本解析、用户`Path`注册表访问等。相比直接猜问题，先跑一次自检会更快定位原因。

## 选择部分工具

如果你并不想安装全部工具，可以指定单独的工具：

```
armup install --tool probe-rs,ninja --root D:\Embedded_Toolchain
```

支持的工具名包括：

```
arm-none-eabi-gcc
clangd
cmake
ninja
probe-rs
xpack-openocd
```

## 选择版本

默认情况下，`ARMUP`会安装最新支持版本。如果你想从最近几个上游版本里手动选择，可以使用：

```
armup install -a --root D:\Embedded_Toolchain --select-versions
```

注意：`--select-versions`需要交互式终端。如果你在脚本、CI 或非交互环境里运行，就不要使用这个选项。

## 安装完成后检查

重新打开终端后，可以逐个检查版本：

```
arm-none-eabi-gcc --version
clangd --version
cmake --version
ninja --version
probe-rs --version
openocd --version
```

如果这些命令都能正常输出版本号，说明工具链和`Path`基本已经配置完成。后续就可以继续安装芯片对应的`Rust target`，然后使用`probe-rs`或`OpenOCD`进行烧录调试。

## 常见问题

- 非交互模式下仍然需要传入`--root`，因为安装根目录属于必须明确的选择。
- 修改`Path`后旧终端不会自动更新，重新打开终端再测试。
- 如果下载失败，先跑`armup doctor`，确认网络、代理和上游 release 解析是否正常。
- 如果不想修改用户`Path`，把`--add-path`换成`--no-add-path`。
- 如果根目录里有旧版本，更新后会保留最新需要的版本，并清理不再使用的旧版本目录。
- 建议不要把安装目录放到系统盘深层路径或带空格的目录里，虽然不一定不能用，但后续脚本和工具调用更容易踩坑。

## 结语

`ARMUP`解决的是一个很现实的问题：在`Windows`上搭建嵌入式工具链经常不是难在某个工具本身，而是难在下载来源、压缩包结构、环境变量和后续更新维护。把这些步骤收敛到一个命令里，可以让新项目启动快很多。

欢迎反馈`bug`。enjoy😊。
