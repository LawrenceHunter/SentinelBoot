FROM rust:bullseye
ENV DEBIAN_FRONTEND noninteractive
ENV TERM xterm

RUN apt update -y && \
    apt install -y qemu-system make wget graphviz && \
    wget https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download/2023.04.29/riscv64-elf-ubuntu-20.04-nightly-2023.04.29-nightly.tar.gz && \
    tar -xvf riscv64-elf-ubuntu-20.04-nightly-2023.04.29-nightly.tar.gz && \
    rm riscv64-elf-ubuntu-20.04-nightly-2023.04.29-nightly.tar.gz && \
    cp riscv/bin/* /usr/bin && \
    rm -rf riscv && \
    rustup override set nightly && \
    rustup target add riscv64gc-unknown-none-elf && \
    cargo install cargo-binutils && \
    cargo install cargo-call-stack && \
    cargo install cargo-geiger && \
    rustup +nightly component add rust-src && \
    rustup component add llvm-tools-preview && \
    rustup component add clippy && \
    apt clean
