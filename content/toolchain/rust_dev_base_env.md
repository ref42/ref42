---
title: '五分钟，搭建Rust嵌入式开发环境'
description: '在 Windows 上安装 VSCode、Visual Studio Community Edition 和 Rust 工具链，完成 Rust 嵌入式开发的基础环境配置。'
weight: 10
slug: rust_dev_base_env
---

## 前言

这是`Rust`开发单片机系列教程的第一个教程。此教程会为大家演示如何配置一个基础的`Rust`开发环境。
<img 
  src="/toolchain-images/letmesee.jpg" 
  alt="let me see reaction image" 
  class="article-image"
/>
需要说明的是，与传统是同`C/CPP`语言开发单片机不同，`Rust`开发单片机只需要配置一个开发环境即可。不再需要使用诸如`Keil-uVision5`，`IAR`, `SES`, `MRS/MRS2`, `STM32CubeIDE`等`IDE`。

本教程主要演示如何在`Windows`平台配置开发环境，如果你是`Linux`用户，配置会更简单，有差异的步骤会单独指出，方便一个教程兼容多个平台。

## 准备清单

| 项目 | 说明 |
| --- | --- |
| 操作系统 | Windows 为主，Linux 可参考差异说明 |
| 编辑器 | VSCode |
| 编译依赖 | Visual Studio Community Edition |
| Rust安装器 | rustup-init.exe |
| 预期结果 | 终端可以正常执行`cargo`、`rustc`、`rustup` |

对于已经尝试过`Rust`开发，觉得`Rust`并没有帮你解决问题，也没有达到预期效果，想要卸载`Rust`的用户，只需要打开终端，输入以下命令，再输入`y`，按下`Enter`，便可卸载`Rust`。
```bash
rustup self uninstall
```
![rustup self uninstall confirmation](/toolchain-images/uninstall_rust.png)

此次需要安装的软件如下：
- [`VSCode`](https://code.visualstudio.com/download#)
- > 代码编辑器，提供现代化的代码编辑体验。
- [`Visual Studio Community Edition`](https://c2rsetup.officeapps.live.com/c2r/downloadVS.aspx?sku=community&channel=stable&version=VS18&source=VSLandingPage&cid=2500:4dbd59610b8e40148050ac727642c374)
- > 提供一些底层的`SDK`和`linker`，使用`MSVC ABI`
- [`rustup-init.exe`](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe)
- > `rust`安装引导程序

若是你的电脑上已经存在了部分软件，例如`VSCode`和`Visual Studio Community Edition`，那就太好了，你只需要做很少的工作即可开始`Rust`开发，不仅是开发`MCU`等。

`VSCode`和`Visual Studio Community Edition`的安装不会难住任何人，特别是本频道的各位，所以大家点击上方链接，直接跳转到下载页进行安装即可。

---

## 安装 VSCode 和 Visual Studio Community Edition

看到这一行的时候，我会默认你已经安装了`VSCode`和`Visual Studio Community Edition`，从这一段之后便是正式的配置环节。

第一步，配置`VSCode`，这一步最好是新建一个`profile`来作为`rust`开发环境，不要与其他的工具链混用。之后再安装以下插件即可。
- `rust analyzer`
- `even better toml`
- `dependi`

第二步，打开`Visual Studio Installer`，这个软件是用于安装`Visual Studio Community Edition`的。

确保你安装了以下这些`组件`。以截图形式给出，自行对照安装即可。

打开`Visual Studio Installer`之后，点击红色箭头指示的`Modify`。
![Visual Studio Installer modify button](/toolchain-images/visual_studio_installer_startup.png)

在弹出的窗口里确保红色箭头指示的这些组件被成功安装。

![Visual Studio desktop development workload selected](/toolchain-images/visual_studio_installer_desktop_dev.png)

![Windows SDK component selected in Visual Studio Installer](/toolchain-images/visual_studio_installer_win_sdk.png)

至此，已经完成`2/3`的进度了。

---

## 安装 Rust

第三步，安装`Rust`。

由于默认安装会将整个`Rust`工具链安装到`C`盘，考虑到部分开发者的`C`可能甚至没有`512GB`，故先配置环境变量，便于将`Rust`工具链安装到其他空间充裕的盘符。

教程里演示的是安装到`D`盘，先在`D`盘新建两个目录，用于安装`Rust`工具链。

---

### 配置环境变量

具体操作步骤：新建文件夹/目录，命名为`RUST`，进入`RUST`文件夹/目录，新建两个目录，分别为`.cargo`和`.rustup`。

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

环境变量配置完成。

双击打开`rustup-init.exe`

可以在终端里看到红色箭头指示的内容，则说明`Rust`工具链安装的环境变量配置成功。

![rustup-init startup options](/toolchain-images/rustup_init_startup.png)

随后，输入`2`，按下`Enter`。
![rustup customize installation menu](/toolchain-images/rustup_init_cusomize_installation.png)
紧接着会弹出确认信息，继续按`Enter`。
![rustup host triple confirmation](/toolchain-images/rustup_init_cusomize_installation_host.png)
随后会弹出确认`toolchain`选择信息，此时输入`nightly`并且按下`Enter`。
![rustup toolchain selection prompt](/toolchain-images/rustup_init_cusomize_installation_toolchain.png.png)
随后还会确认`profile`，按`Enter`即可。
![rustup profile selection prompt](/toolchain-images/rustup_init_cusomize_installation_profile.png)
最后会询问是否要`modify`环境变量，也就是采用我们之前配置的环境变量。输入`y`并且按下`Enter`。
![rustup PATH modification prompt](/toolchain-images/rustup_init_cusomize_installation_path.png)
按下`Enter`之后，会再次出现以下提示信息。
![rustup installation confirmation screen](/toolchain-images/rustup_init_cusomize_installation_final.png)
此时，只需要按下`Enter`即可。
![rustup installation progress](/toolchain-images/rustup_init_cusomize_installation_process.png)
看到终端里的`Rust is installed now. Great!`，则说明此次安装圆满完成。
![Rust installed successfully message](/toolchain-images/OK.png)

以上步骤对于新手来说，可能稍显繁琐，但这是实践下来，个人认为比较满意的安装方式。

如果你使用的是`Linux`，只需要你安装了`gcc`，然后，直接在官网复制安装命令，进行安装即可，环境配置的操作是一样的，无非是将其写进`.bashrc`，但大多数时候默认安装即可。

至此，`Windows`平台的`Rust`开发环境就算搭建完成了，这个环境可以理解为一个基础环境。你可以选择用其开发桌面应用、命令行工具，也可以在其基础上添加一些工具，用来做嵌入式开发。

---

## 资料推荐

若想要进一步学习`Rust`，也可以给大家推荐一些资源。
- [B站杨旭老师的视频](https://www.bilibili.com/video/BV1hp4y1k7SV?spm_id_from=333.1387.collection.video_card.click)

<img
src="/toolchain-images/yangxu_bilibili.png"
alt="rust_book"
class="article-image"
/>

- [Rust圣经](https://doc.rust-lang.org/book/title-page.html)
- >这本书可以结合视频一起看。

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

## 常见问题

- 终端找不到`cargo`时，重新打开终端，或检查`CARGO_HOME`和`PATH`是否已经生效。
- 安装卡住时，优先确认网络代理、杀毒软件和系统权限是否影响下载。
- 后续开发某个芯片平台前，还需要继续安装对应平台的`target`、烧录工具和调试工具。
