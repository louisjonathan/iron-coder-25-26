#!/bin/bash
set -e
if command -v rustc >/dev/null 2>&1; then
    echo "Rust is already installed. Skipping install of Rust, installing dependencies only"
else
    echo "Rust is not installed. Installing Rust and dependencies"
    wget https://sh.rustup.rs -O rustup-init.sh
    sh rustup-init.sh -y --no-modify-path --default-toolchain stable
    export PATH="$HOME/.cargo/bin:$PATH"
    rm rustup-init.sh
fi
cd "$(dirname "$0")"
rustup component add clippy rustfmt
rustup update stable
rustup update nightly
rustup target add thumbv6m-none-eabi
rustup target add riscv32imac-unknown-none-elf
cargo install cargo-generate espflash ravedude probe-rs-tools
cargo build --release
