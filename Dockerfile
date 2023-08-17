FROM debian:bullseye-slim AS toolchain
WORKDIR /src
ENV DEBIAN_FRONTEND noninteractive
RUN apt update -y && \
    apt install -y wget && \
    apt clean
RUN wget https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download/2023.05.27/riscv64-glibc-ubuntu-20.04-nightly-2023.05.27-nightly.tar.gz && \
    tar -xvf riscv64-glibc-ubuntu-20.04-nightly-2023.05.27-nightly.tar.gz && \
    rm riscv64-glibc-ubuntu-20.04-nightly-2023.05.27-nightly.tar.gz

# Split more than it should to prevent Pi crashes
FROM rust:bullseye AS rust
ENV DEBIAN_FRONTEND noninteractive
ENV TERM xterm
COPY --from=toolchain /src/riscv/bin /usr/bin
RUN rustup override set nightly
RUN rustup target add riscv64gc-unknown-none-elf
RUN cargo install -j1 cargo-binutils
RUN cargo install -j1 cargo-call-stack
RUN cargo install -j1 cargo-geiger
RUN rustup +nightly component add rust-src
RUN rustup component add llvm-tools-preview
RUN rustup component add clippy
RUN cargo install -j1 hyperfine
RUN apt update -y && \
    apt install -y qemu-system make wget graphviz python3-pip && \
    apt clean
RUN pip3 install pyfiglet
