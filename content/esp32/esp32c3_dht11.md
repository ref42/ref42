---
title: 'ESP32C3读取DHT11温湿度信息'
description: '使用 Rust 在 ESP32-C3 上读取 DHT11 温湿度传感器，并通过 cargo run 完成编译、烧录和终端输出。'
weight: 20
slug: esp32c3_dht11
---

## 前言和实验结果
> 跳过`Hello World`，即点亮`LED`，直接实战读取`DHT11`温湿度传感器。

## 实验信息

| 项目 | 内容 |
| --- | --- |
| 目标芯片 | ESP32-C3 |
| 传感器 | DHT11 温湿度模块 |
| 数据引脚 | GPIO0 |
| 前置环境 | [Rust基础环境](/toolchain/rust_dev_base_env/) + [ESP32 Rust环境](/toolchain/esp32-dev-setup/) |
| 预期结果 | 终端持续打印温湿度数据 |

![exhausted reaction image](/memes/exhausted.jpg)

最后实现的效果如下。

![DHT11 temperature and humidity output in terminal](/esp32/esp32c3/dht11/dht11-measuring-result.png)

> 如上图，可以在终端`Terminal`里直接看到`DHT11`测量到的温湿度信息，无需额外的串口。终端里也支持中文信息输出，感兴趣可以自己上手实测。

## 使用 esp-generate 生成模板工程

> 由于过程具有重复性，故不再重复演示，请点击跳转[使用esp-generate生成一个ESP32初始工程](/esp32/esp-generate-template/)

## 打开工程
- 导入需要用到的`crate`，在工程根目录下打开终端，输入以下命令，并按下回车（`Enter`）

```rust
cargo add esp32-dht11-rs
```
在`main.rs`里输入以下代码
- 在头部的位置，先引入需要使用的`crate`

```rust
use esp32_dht11_rs::{DHT11};
use esp_hal::delay::Delay;
```
- 接着，来到`main`函数里

```rust
let delay = Delay::new();
let mut dht11 = DHT11::new(peripherals.GPIO0, delay);
let mut dht11_read_counter = 1;
```
- 再将以下内容复制到`loop`函数里

```rust
match dht11.read() {
            Ok(m) => println!("DHT11 TEMP is: {}℃ DHT11 HUMI is:{}% measuring for [NO.{:?}] time", m.temperature, m.humidity, dht11_read_counter),
            Err(error) => println!("error occurred: {:?}", error),
        }
        Timer::after_millis(2000).await;
        dht11_read_counter += 1;
```
- 断电前提下，连接硬件
```bash
# esp32c3   # dht11
gpio0   ->    dat
vcc     ->    vcc
gnd     ->    gnd
```

- 连接好硬件之后，将开发板连接到电脑，回到终端，输入以下命令
> 需要自行安装`CH34x`的驱动，这一步不做说明，自行安装即可。
```bash
cargo run --release
```
可以看到终端里在进行编译`compiling`。

![terminal compiling ESP32-C3 DHT11 firmware](/esp32/esp32c3/dht11/terminal-compiling.png)

完成之后，会自动进行烧录，期间不需要按开发板上的任何按键（试过很多板子，都不需要按`BOOT`）。

![terminal showing ESP32-C3 firmware compilation complete](/esp32/esp32c3/dht11/terminal-compiling-done.png)

烧录完成后，正常在终端里打印信息，若是用手握住`DHT11`模块，湿度会瞬间上升，温度也会开始缓慢上升（假定你手现在并没有进行过冷冻处理，是正常体温。），说明模块工作正常，程序正常运行。

![DHT11 readings printed after flashing ESP32-C3](/esp32/esp32c3/dht11/dht11-running-ok.png)

## 常见问题

- 终端没有数据时，先检查`GPIO0`、`VCC`、`GND`是否接反，并确认传感器模块电压与开发板兼容。
- `cargo run --release`无法烧录时，检查`espflash`是否安装成功，以及开发板串口是否被其他软件占用。
- 编译报找不到`esp32_dht11_rs`时，回到工程根目录重新执行`cargo add esp32-dht11-rs`。
