---
title: 'Read DHT11 temperature and humidity on ESP32-C3'
description: 'Read a DHT11 temperature and humidity sensor with Rust on ESP32-C3, then build, flash, and monitor it with cargo run.'
weight: 20
slug: esp32c3_dht11
date: "2026-07-11"
tags: [esp32, sensor]
---

## Overview and result
> Skip the usual `Hello World` LED example and read a `DHT11` temperature and humidity sensor directly.

## Experiment details

| Item | Details |
| --- | --- |
| Target chip | ESP32-C3 |
| Sensor | DHT11 temperature and humidity module |
| Data pin | GPIO0 |
| Prerequisites | [Base Rust environment](/toolchain/rust_dev_base_env/) + [ESP32 Rust environment](/toolchain/esp32-dev-setup/) |
| Expected result | The terminal continuously prints temperature and humidity data |

![exhausted reaction image](/memes/exhausted.jpg)

The final output looks like this.

![DHT11 temperature and humidity output in terminal](/esp32/esp32c3/dht11/dht11-measuring-result.png)

> The `DHT11` readings appear directly in the terminal without an extra serial monitor.

## Generate the template project

> The process is the same as the previous guide, so follow [Create an ESP32 starter project with esp-generate](/esp32/esp-generate-template/).

## Open the project
- Add the required `crate` from the project root:

```rust
cargo add esp32-dht11-rs
```
Add the following code to `main.rs`.
- Import the required `crate` at the top:

```rust
use esp32_dht11_rs::{DHT11};
use esp_hal::delay::Delay;
```
- Then initialize the sensor in `main`:

```rust
let delay = Delay::new();
let mut dht11 = DHT11::new(peripherals.GPIO0, delay);
let mut dht11_read_counter = 1;
```
- Copy this block into the `loop`:

```rust
match dht11.read() {
            Ok(m) => println!("DHT11 TEMP is: {}℃ DHT11 HUMI is:{}% measuring for [NO.{:?}] time", m.temperature, m.humidity, dht11_read_counter),
            Err(error) => println!("error occurred: {:?}", error),
        }
        Timer::after_millis(2000).await;
        dht11_read_counter += 1;
```
- With the board powered off, connect the hardware:
```bash
# esp32c3   # dht11
gpio0   ->    dat
vcc     ->    vcc
gnd     ->    gnd
```

- Connect the board to the computer and run:
> Install the `CH34x` driver separately if your board requires it.
```bash
cargo run --release
```
The terminal will show the `compiling` step.

![terminal compiling ESP32-C3 DHT11 firmware](/esp32/esp32c3/dht11/terminal-compiling.png)

Flashing starts automatically; you normally do not need to press any board button, including `BOOT`.

![terminal showing ESP32-C3 firmware compilation complete](/esp32/esp32c3/dht11/terminal-compiling-done.png)

After flashing, readings should print in the terminal. Holding the `DHT11` module will quickly raise humidity and slowly raise temperature, confirming that the sensor and program work.

![DHT11 readings printed after flashing ESP32-C3](/esp32/esp32c3/dht11/dht11-running-ok.png)

## Troubleshooting

- If the terminal is empty, check that `GPIO0`, `VCC`, and `GND` are wired correctly and that the sensor voltage matches the board.
- If `cargo run --release` cannot flash, verify that `espflash` is installed and that no other program owns the board's serial port.
- If compilation cannot find `esp32_dht11_rs`, return to the project root and run `cargo add esp32-dht11-rs` again.
