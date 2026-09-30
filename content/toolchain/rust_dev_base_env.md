---
title: 'Set up a Rust embedded environment in five minutes'
description: 'Install VSCode, Visual Studio Community Edition, and the Rust toolchain on Windows for embedded development.'
weight: 10
slug: rust_dev_base_env
date: "2026-07-11"
tags: [toolchain, windows]
---

## Introduction

This is the first guide in the Rust microcontroller series. It shows how to configure a basic `Rust` development environment.
<img 
  src="/toolchain-images/letmesee.jpg" 
  alt="let me see reaction image" 
  class="article-image"
/>
Unlike traditional microcontroller development with `C/C++`, `Rust` needs one general development environment. You do not need IDEs such as `Keil-uVision5`, `IAR`, `SES`, `MRS/MRS2`, or `STM32CubeIDE`.

This guide uses `Windows`. The setup is simpler on `Linux`; platform differences are called out where needed.

## Checklist

| Item | Details |
| --- | --- |
| Operating system | Windows (Linux notes are included where relevant) |
| Editor | VSCode |
| Build dependency | Visual Studio Community Edition |
| Rust installer | rustup-init.exe |
| Expected result | The terminal can run `cargo`, `rustc`, and `rustup` |

If you have tried `Rust` and want to remove it, open a terminal, run the command below, type `y`, and press `Enter`.
```bash
rustup self uninstall
```
![rustup self uninstall confirmation](/toolchain-images/uninstall_rust.png)

Install the following software:
- [`VSCode`](https://code.visualstudio.com/download#)
- > A code editor with a modern editing experience.
- [`Visual Studio Community Edition`](https://c2rsetup.officeapps.live.com/c2r/downloadVS.aspx?sku=community&channel=stable&version=VS18&source=VSLandingPage&cid=2500:4dbd59610b8e40148050ac727642c374)
- > Provides the low-level `SDK` and `linker` used by the `MSVC ABI`.
- [`rustup-init.exe`](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe)
- > The `rust` installation bootstrapper.

If `VSCode` and `Visual Studio Community Edition` are already installed, you can start `Rust` development with only a few additional steps.

Install `VSCode` and `Visual Studio Community Edition` from the links above.

---

## Install VSCode and Visual Studio Community Edition

The steps below assume that `VSCode` and `Visual Studio Community Edition` are installed.

First, configure `VSCode`. Create a dedicated `profile` for `Rust` so it does not mix with other toolchains, then install these extensions:
- `rust analyzer`
- `even better toml`
- `dependi`

Next, open `Visual Studio Installer`, which manages `Visual Studio Community Edition`.

Make sure the following `components` are installed. Compare the screenshots with your installer.

Open `Visual Studio Installer` and click the `Modify` button highlighted in the screenshot.
![Visual Studio Installer modify button](/toolchain-images/visual_studio_installer_startup.png)

In the dialog, make sure the highlighted components are selected and installed.

![Visual Studio desktop development workload selected](/toolchain-images/visual_studio_installer_desktop_dev.png)

![Windows SDK component selected in Visual Studio Installer](/toolchain-images/visual_studio_installer_win_sdk.png)

Two of the three setup stages are complete.

---

## Install Rust

The third stage installs `Rust`.

The default installer places the entire `Rust` toolchain on the `C` drive. Set environment variables first if you want to use another drive.

This guide uses the `D` drive. Create two directories there for the `Rust` toolchain.

---

### Configure environment variables

Create a directory named `RUST`, open it, and create `.cargo` and `.rustup` directories inside.

```bash
# variable name
RUSTUP_HOME
# variable value
D:\RUST\.rustup

```
![RUSTUP_HOME environment variable path](/toolchain-images/rustup_env_path.png)

```bash
# variable name
CARGO_HOME
# variable value
D:\RUST\.cargo

```
![CARGO_HOME environment variable path](/toolchain-images/cargo_env_path.png)

The environment variables are configured.

Double-click `rustup-init.exe`.

The highlighted output in the terminal confirms that the environment variables are ready for the `Rust` toolchain.

![rustup-init startup options](/toolchain-images/rustup_init_startup.png)

Enter `2` and press `Enter`.
![rustup customize installation menu](/toolchain-images/rustup_init_cusomize_installation.png)
When the confirmation appears, press `Enter` again.
![rustup host triple confirmation](/toolchain-images/rustup_init_cusomize_installation_host.png)
When the `toolchain` selection appears, enter `nightly` and press `Enter`.
![rustup toolchain selection prompt](/toolchain-images/rustup_init_cusomize_installation_toolchain.png.png)
Accept the `profile` selection with `Enter`.
![rustup profile selection prompt](/toolchain-images/rustup_init_cusomize_installation_profile.png)
When asked whether to `modify` the environment variables, enter `y` and press `Enter` to use the values configured earlier.
![rustup PATH modification prompt](/toolchain-images/rustup_init_cusomize_installation_path.png)
Another prompt appears after you press `Enter`.
![rustup installation confirmation screen](/toolchain-images/rustup_init_cusomize_installation_final.png)
Press `Enter` to continue.
![rustup installation progress](/toolchain-images/rustup_init_cusomize_installation_process.png)
The message `Rust is installed now. Great!` confirms a successful installation.
![Rust installed successfully message](/toolchain-images/OK.png)

These steps may feel long for beginners, but they provide a predictable installation layout.

On `Linux`, install `gcc` and run the official installation command. The environment variables work the same way, usually by adding them to `.bashrc`; the defaults are often sufficient.

The basic `Rust` environment on `Windows` is now ready. Use it for desktop applications and command-line tools, or add platform tools for embedded development.

---

## Recommended resources

For further `Rust` learning, these resources are useful:
- [Yang Xu's video course](https://www.bilibili.com/video/BV1hp4y1k7SV?spm_id_from=333.1387.collection.video_card.click)

<img
src="/toolchain-images/yangxu_bilibili.png"
alt="rust_book"
class="article-image"
/>

- [The Rust Book](https://doc.rust-lang.org/book/title-page.html)
- > Read this book alongside the video course.

<img
src="/toolchain-images/rust_book.png"
alt="rust_book"
class="article-image"
/>

- [Web Search and AI is ALL U Need.](https://www.google.com/)

<img 
  src="/toolchain-images/cool.jpg" 
  alt="cool" 
  class="article-image"
/>

## Troubleshooting

- If the terminal cannot find `cargo`, open a new terminal or check that `CARGO_HOME` and `PATH` are active.
- If installation stalls, check the network proxy, antivirus software, and system permissions.
- Before developing for a chip platform, install its `target`, flashing tool, and debugger.
