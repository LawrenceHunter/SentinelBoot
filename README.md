# WIP

# Docker

### Setup
https://docs.docker.com/get-docker/

### Build
```bash
make
```

### Run
```bash
make qemu
```

## Run bare metal
```bash
make
# Flash bootloader.img to sd card
```

# Host

### Setup
```bash
# You will need a riscv toolchain
# https://github.com/riscv-collab/riscv-gnu-toolchain

# Skip if rust is already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Skip if QEMU already installed
# Apt
sudo apt install qemu-system
# Pacman
sudo pacman -S qemu-system-riscv

rustup toolchain install nightly
rustup override set nightly
rustup target add riscv64gc-unknown-none-elf
cargo install cargo-binutils
rustup component add llvm-tools-preview
```
### Build
```bash
make
```

### Run
```bash
make qemu
```

## Run bare metal
```bash
make
# Flash bootloader.img to sd card
```
