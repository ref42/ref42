---
title: 'svd2rust'
description: '使用 svd2rust 从 STM32 SVD 文件生成 PAC，并完成寄存器级点灯实验。'
weight: 10
slug: svd2rust
---

## 简介

`svd2rust`，顾名思义，可以直接从`SVD(System View Description)`文件生成`Rust`代码，经过`svd2rust`生成的`Rust`代码其实是一个`PAC(Peripheral Access Crate)`，初次接触的时候，可以将其理解为最贴近寄存器的一个库。

本次教程会优先演示如何从`svd`到点亮`led`的完整流程，教程的末尾会给出两个脚本，结合`svd`文件，可以一键完成点灯。

## 如何使用

1. 获取`SVD`文件，这一步的途径有很多：某些`IDE`的文件资源；`ARM`官网；芯片官网。本教程演示的芯片为`STM32F405RGT6`，可以直接在官网搜索到其`SVD`文件。

2. 安装必要的工具
```bash
cargo install svd2rust 
cargo install flip-link
cargo install form
```

3. 使用`svd2rust`生成`pac`

这部分其实也可以直接在`svd2rust`的文档里找到，以下直接给出具体命令。

```bash
svd2rust -i STM32F405.svd

rm -rf src

form -i lib.rs -o src/ && rm lib.rs

cargo fmt
```

单看以上命令，或许可以大概看懂，但是有一些前置条件是默认用户已经掌握的。例如需要在一个`lib`工程里执行以上操作，使用以下命令创建一个`lib`工程。

```bash
cargo new stm32f405_pac --lib
```

随即，将以下依赖添加到`lib`工程的`Cargo.toml`文件里

```toml
[dependencies]
critical-section = { version = "1.0", optional = true }
cortex-m = "0.7.6"
cortex-m-rt = { version = "0.6.13", optional = true }
vcell = "0.1.2"

[features]
rt = ["cortex-m-rt/device"]
```

完成以上命令之后，就得到了一个默认的`pac`，按理来说就可以直接开始调用这个`pac`完成点灯了。

但由于工具链的不断更新，以及依赖的更新，可能导致部分不兼容，所以，需要对默认生成的`pac`进行微调，还需要修改依赖版本。

## 开始点灯

`pac`里存在的部分问题会在这一部分做调整。

1. 创建一个`bin`工程

```bash
cargo new blinky --bin 
```

2. 配置工程，适配嵌入式开发
- 新增文件：`.cargo/config.toml`
```toml

[build]
target = "thumbv7em-none-eabihf"

[target.thumbv7em-none-eabihf]
rustflags = [
    "-C", "link-arg=-Tlink.x",
    "-C", "linker=flip-link",
]
runner = "probe-rs run --chip STM32F405RG"
```
- 新增文件：`.vscode/settings.json`
```json
{
    "rust-analyzer.check.allTargets": false
}

```
- 新增文件：'memory.x'

```text
MEMORY {
  FLASH : ORIGIN = 0x08000000, LENGTH = 1024K
  RAM   : ORIGIN = 0x20000000, LENGTH = 128K
  CCMRAM : ORIGIN = 0x10000000, LENGTH = 64K
}
```
- 添加依赖：`Cargo.toml`
```toml
[dependencies]
stm32f405_pac ={ path = "../stm32f405_pac", features = ["critical-section", "rt"]}
cortex-m ={ version = "0.7.7", features = ["critical-section-single-core"]}
cortex-m-rt ={ version = "0.7.5"}
panic-halt ={ version = "1.0.0"}
```
至此，工程配置完成，但还是会有错误，这部分错误来自`pac`，以下对`pac`进行微调。

3. 微调`pac`

每个`fix`里分为两部分，第一部分是默认的`pac`，第二部分是修改过后的`pac`。

总结下来就是在适当的位置加了一些`unsafe`关键字。可以直接在`vscode`里跳转添加。

- fix 1
```rust
#[doc = "Cryptographic processor"]
pub mod cryp;
#[no_mangle]
static mut DEVICE_PERIPHERALS: bool = false;
#[doc = r" All the peripherals."]
#[allow(non_snake_case)]
pub struct Peripherals
```
```rust
#[doc = "Cryptographic processor"]
pub mod cryp;
#[unsafe(no_mangle)]
static mut DEVICE_PERIPHERALS: bool = false;
#[doc = r" All the peripherals."]
#[allow(non_snake_case)]
pub struct Peripherals
```

- fix 2
```rust
extern "C"
```
```rust
unsafe extern "C"
```

-fix 3
```rust
#[link_section = ".vector_table.interrupts"]
#[no_mangle]
```
```rust
#[unsafe(link_section = ".vector_table.interrupts")]
#[unsafe(no_mangle)]
```

完成以上的微调之后，应该就没有报错了。可以开始编写代码点灯了。

4. 将以下内容添加到`src/main.rs`里
```rust
#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f405_pac::Peripherals;

fn delay(cycles: u32) {
    for _ in 0..cycles {
        asm::nop();
    }
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    dp.rcc.ahb1enr().modify(|_, w| w.gpioben().set_bit());
    dp.gpiob
        .moder()
        .modify(|_, w| unsafe { w.moder13().bits(0b01) });

    loop {
        dp.gpiob.bsrr().write(|w| w.bs13().set_bit());
        delay(50_000);
        dp.gpiob.bsrr().write(|w| w.br13().set_bit());
        delay(50_000);
    }
}
```

5. 烧录代码，查看效果

使用`ST-Link`或者`DAP-Link`连接开发板到电脑，执行以下命令即可完成编译和烧录。
```bash
cargo run --release
```
如果使用`JLink`，则需要使用`zadig`替换`USB`驱动，推荐使用`ST-Link`或者`DAP-Link`。

之后的教程会使用`JLink`配合`Ozone`进行调试，更加方便。

## 脚本介绍

一方面考虑到如果每次都需要执行这么多命令，才能点亮一个`LED`未免有些过于繁琐，考虑到大部分的命令，其实都是固定的，何不将其写到脚本里，一键执行即可。

另一方面考虑到部分读者/观众可能确实感兴趣，但是跟完了以上的流程后，却并不能完成点灯，未免会挫败感，不利于后续学习，故将其封装为两个脚本，分别为`pac.sh`和`blinky.sh`，读者/观众只需要获取到`svd`文件，并将这三个文件放置到同级目录下即可一键生成。

## 脚本使用

- `pac.sh`脚本
```bash
#!/bin/bash

# install some tools needed
cargo install svd2rust 
cargo install flip-link
cargo install form

cargo new stm32f405_pac --lib

echo "lib created"

cp -r ./STM32F405.svd ./stm32f405_pac

echo "svd copied"

cd stm32f405_pac

# svd2rust part
svd2rust -i ./STM32F405.svd 

rm -rf src

form -i lib.rs -o src/ && rm lib.rs

cargo fmt

# add dependencies to pac/Cargo.toml
sed -i '/^\[dependencies\]$/d' Cargo.toml
cat >> Cargo.toml << 'EOF'
[dependencies]
critical-section = { version = "1.0", optional = true }
cortex-m = "0.7.7"
cortex-m-rt = { version = "0.7.5", optional = true }
vcell = "0.1.2"

[features]
rt = ["cortex-m-rt/device"]

EOF

echo "done"
```

- `blinky.sh`
```bash
#!/bin/bash

cargo new blinky --bin

cd blinky

mkdir -p .cargo

cat > .cargo/config.toml << 'EOF'

[build]
target = "thumbv7em-none-eabihf"

[target.thumbv7em-none-eabihf]
rustflags = [
    "-C", "link-arg=-Tlink.x",
    "-C", "linker=flip-link",
]
runner = "probe-rs run --chip STM32F405RG"

EOF

sed -i '/^\[dependencies\]$/d' Cargo.toml
cat >> Cargo.toml << 'EOF'
[dependencies]
stm32f405_pac ={ path = "../stm32f405_pac", features = ["critical-section", "rt"]}
cortex-m ={ version = "0.7.7", features = ["critical-section-single-core"]}
cortex-m-rt ={ version = "0.7.5"}
panic-halt ={ version = "1.0.0"}

EOF

touch memory.x

cat > memory.x << 'EOF'
MEMORY {
  FLASH : ORIGIN = 0x08000000, LENGTH = 1024K
  RAM   : ORIGIN = 0x20000000, LENGTH = 128K
  CCMRAM : ORIGIN = 0x10000000, LENGTH = 64K
}

EOF

mkdir -p .vscode

cat > .vscode/settings.json << 'EOF'
{
    "rust-analyzer.check.allTargets": false
}
EOF

echo "done"
```

先执行`./pac.sh`，再执行`./blinky.sh`。

## 打开脚本生成的工程进行编译

由于脚本并没有对`pac`进行微调，所以还需要使用`vscode`打开`blinky`工程，点击跳转到`error`的位置，添加`unsafe`关键字。

## 脚本使用注意事项
脚本仅针对`STM32F405RG`芯片，若你是用的是其他芯片，需要提供相应的`svd`文件，以及修改脚本里的一些关键配置。

- `.cargo/config.toml`

`target`需要与芯片架构对应，先使用以下命令安装对应的`target`。

```bash
# M0/M0+内核芯片，常见芯片：STM32F0xx，PY32F0xx，....
rustup target add thumbv6m-none-eabi

# M3内核芯片，常见芯片：STM32F1xx，...
rustup target add thumbv7m-none-eabi

# M4F和M7F内核芯片，F意指带有浮点运算，常见芯片：STM32F4xx，STM32G4xx，STM32H7xx，...
rustup target add thumbv7em-none-eabihf

# M33F内核芯片，常见芯片：STM32U5xx，...
rustup target add thumbv8m.main-none-eabihf
```

`--chip STM32F405RG` 也需要适配自己的型号，例如：`--chip STM32F103C8`

- `memory.x`
```text
MEMORY {
  FLASH : ORIGIN = 0x08000000, LENGTH = 1024K
  RAM   : ORIGIN = 0x20000000, LENGTH = 128K
  CCMRAM : ORIGIN = 0x10000000, LENGTH = 64K
}
```
只需要修改`LENGTH`相应的大小即可，`ST`系列单片机的起始地址一般都是`0x08000000`和`0x20000000`。

对于没有`CCRAM`的型号，直接删除`CCRAM一行即可`。

## 总结

此次教程为读者/观众演示了从寄存器层面点亮`led`的完整流程，并且编写了一键运行的脚本帮助读者/观众理解整个流程。

此次教程的目的在于为读者/观众演示`Rust`嵌入式开发的底层控制能力，读者/观众可以根据自己的能力来评估是否还需要继续深入了解学习更加底层的知识。

推荐读者/观众先在`ST`系列的单片机上跑通整个流程，再去考虑国产芯片，一些国产芯片的`svd`文件规范性实在不敢恭维。

许多国产芯片，类似`py32`和`ch32`以及`hpm`等，都有国内开发者已经编写了大部分的`hal`，甚至还添加了`embassy`的支持，这无疑是一件好事，初学者也不必纠结非要自己维护一套`hal`，也可以考虑去这些仓库进行`pr`或者`issue`。

## 建议

此篇教程另一个目的是为了向读者/观众展示“长江水从何处来”，至于实际生产和学习的时候，更加推荐去使用一些别人已经在`pac`基础上封装的`hal`。例如[`rtic`](https://rtic.rs/2/book/en/)和[`embassy`](https://embassy.dev/)以及诸多的`xxx_hal`

## 参考资料
- [svd2rust](https://docs.rs/svd2rust/latest/svd2rust/)
- [Rust Embedded](https://rust-lang.org/what/embedded/)
