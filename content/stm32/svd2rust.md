---
title: 'svd2rust'
description: 'Generate a PAC from an STM32 SVD file with svd2rust, then run a register-level LED experiment.'
weight: 10
slug: svd2rust
date: "2026-07-11"
tags: [stm32, svd2rust]
---

## Introduction

As its name suggests, `svd2rust` generates `Rust` code directly from an `SVD (System View Description)` file. The generated code is a `PAC (Peripheral Access Crate)`, which you can think of as a library close to the hardware registers.

This guide walks through the complete path from an `svd` file to a blinking `LED`. Two scripts at the end automate the process.

## Usage

1. Obtain an `SVD` file from an `IDE` installation, the `ARM` website, or the chip vendor. This guide uses `STM32F405RGT6`; its `SVD` file is available from the vendor.

2. Install the required tools
```bash
cargo install svd2rust 
cargo install flip-link
cargo install form
```

3. Generate a `pac` with `svd2rust`

The same procedure is covered in the `svd2rust` documentation. The commands are shown here for convenience.

```bash
svd2rust -i STM32F405.svd

rm -rf src

form -i lib.rs -o src/ && rm lib.rs

cargo fmt
```

The commands assume a few prerequisites. For example, run them inside a `lib` project, which you can create with:

```bash
cargo new stm32f405_pac --lib
```

Add these dependencies to the `lib` project's `Cargo.toml`:

```toml
[dependencies]
critical-section = { version = "1.0", optional = true }
cortex-m = "0.7.6"
cortex-m-rt = { version = "0.6.13", optional = true }
vcell = "0.1.2"

[features]
rt = ["cortex-m-rt/device"]
```

The commands produce a default `pac` that can be used to control the LED.

Toolchain and dependency updates can introduce incompatibilities, so the generated `pac` and dependency versions may need small adjustments.

## Blink the LED

This section fixes the issues in the generated `pac` and builds a blinky application.

1. Create a `bin` project

```bash
cargo new blinky --bin 
```

2. Configure the project for embedded development
- Add `.cargo/config.toml`:
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
- Add `.vscode/settings.json`:
```json
{
    "rust-analyzer.check.allTargets": false
}

```
- Add `memory.x`:

```text
MEMORY {
  FLASH : ORIGIN = 0x08000000, LENGTH = 1024K
  RAM   : ORIGIN = 0x20000000, LENGTH = 128K
  CCMRAM : ORIGIN = 0x10000000, LENGTH = 64K
}
```
- Add these dependencies to `Cargo.toml`:
```toml
[dependencies]
stm32f405_pac ={ path = "../stm32f405_pac", features = ["critical-section", "rt"]}
cortex-m ={ version = "0.7.7", features = ["critical-section-single-core"]}
cortex-m-rt ={ version = "0.7.5"}
panic-halt ={ version = "1.0.0"}
```
The project is configured, but errors may remain in the `pac`; fix them as follows.

3. Adjust the `pac`

Each fix shows the generated `pac` first and the corrected version second.

In summary, add `unsafe` in the indicated places. You can jump to each location in `vscode`.

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

- fix 3
```rust
#[link_section = ".vector_table.interrupts"]
#[no_mangle]
```
```rust
#[unsafe(link_section = ".vector_table.interrupts")]
#[unsafe(no_mangle)]
```

After these adjustments, the errors should be gone and you can write the LED application.

4. Add the following to `src/main.rs`:
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

5. Flash the code and observe the result

Connect the board with `ST-Link` or `DAP-Link`, then run the command below to build and flash it.
```bash
cargo run --release
```
If you use `JLink`, replace its `USB` driver with `zadig`. `ST-Link` or `DAP-Link` is recommended.

Later guides use `JLink` with `Ozone` for debugging.

## Script overview

Running this many commands for every `LED` is tedious, and most commands are fixed. Putting them in scripts makes the process one step.

The two scripts, `pac.sh` and `blinky.sh`, also help readers who want to try the process without troubleshooting every manual step. Place them and the `svd` file in one directory to generate the projects.

## Use the scripts

- `pac.sh`
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

Run `./pac.sh` first, then `./blinky.sh`.

## Build the generated project

The scripts do not adjust the `pac`. Open the `blinky` project in `vscode`, jump to each `error`, and add the required `unsafe` keyword.

## Script notes
The scripts target `STM32F405RG`. For another chip, provide its `svd` file and update the relevant configuration.

- `.cargo/config.toml`

The `target` must match the chip architecture. Install the appropriate `target` with one of these commands:

```bash
# M0/M0+ core; common chips: STM32F0xx, PY32F0xx, ...
rustup target add thumbv6m-none-eabi

# M3 core; common chips: STM32F1xx, ...
rustup target add thumbv7m-none-eabi

# M4F and M7F cores; F means hardware floating point. Common chips: STM32F4xx, STM32G4xx, STM32H7xx, ...
rustup target add thumbv7em-none-eabihf

# M33F core; common chips: STM32U5xx, ...
rustup target add thumbv8m.main-none-eabihf
```

Adapt `--chip STM32F405RG` to your model, for example `--chip STM32F103C8`.

- `memory.x`
```text
MEMORY {
  FLASH : ORIGIN = 0x08000000, LENGTH = 1024K
  RAM   : ORIGIN = 0x20000000, LENGTH = 128K
  CCMRAM : ORIGIN = 0x10000000, LENGTH = 64K
}
```
Change the `LENGTH` values to match the device. `ST` microcontrollers commonly use `0x08000000` and `0x20000000` as start addresses.

For a model without `CCRAM`, delete the `CCMRAM` line.

## Summary

This guide demonstrates the complete register-level LED workflow and provides scripts that run it with one command.

The goal is to show the low-level control available in `Rust` embedded development. Decide how deeply to study the lower layers based on your needs.

Run the workflow on an `ST` device first. SVD quality varies widely among some other vendors.

Many other chips, including `py32`, `ch32`, and `hpm`, already have community-maintained `hal` crates and sometimes `embassy` support. Beginners can use those crates instead of maintaining a `hal`, and contribute with a `pr` or `issue` when useful.

## Recommendation

This guide shows where the abstractions come from. For production and day-to-day learning, prefer a maintained `hal` built on top of the `pac`, such as [`rtic`](https://rtic.rs/2/book/en/), [`embassy`](https://embassy.dev/), or one of the many `xxx_hal` crates.

## References
- [svd2rust](https://docs.rs/svd2rust/latest/svd2rust/)
- [Rust Embedded](https://rust-lang.org/what/embedded/)
