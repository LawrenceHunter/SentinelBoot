# Copyright (c) 2023-2024 Lawrence Hunter <lawrence.hunter@outlook.com>

# ---------------------------------------------------------------------------- #
#                             Toolchain base image                             #
# ---------------------------------------------------------------------------- #

FROM debian:bullseye-slim AS toolchain
WORKDIR /src
ENV DEBIAN_FRONTEND noninteractive
RUN apt update -y && \
    apt install -y \
    binutils build-essential libtool texinfo \
    gzip zip unzip patchutils curl git python3 python3-pip \
    make cmake ninja-build automake bison flex gperf \
    grep sed gawk python bc wget libgcrypt20-dev \
    zlib1g-dev libexpat1-dev libmpc-dev autoconf \
    libglib2.0-dev libfdt-dev libpixman-1-dev && \
    apt clean
RUN mkdir qemu_bin

RUN git clone https://github.com/qemu/qemu.git && \
    cd qemu && \
    ./configure --target-list=riscv64-softmmu \
        --prefix=/src/qemu_bin && \
    make -j$(nproc) && \
    make install

RUN wget "https://github.com/riscv-collab/riscv-gnu-toolchain/releases/"\
"download/2023.05.27/riscv64-glibc-ubuntu-20.04-nightly"\
"-2023.05.27-nightly.tar.gz" && \
    tar -xvf riscv64-glibc-ubuntu-20.04-nightly-2023.05.27-nightly.tar.gz && \
    rm riscv64-glibc-ubuntu-20.04-nightly-2023.05.27-nightly.tar.gz


# ---------------------------------------------------------------------------- #
#                 Minimal rust build image for reduced overhead                #
# ---------------------------------------------------------------------------- #

FROM rust:bullseye AS rust-minimal
ENV DEBIAN_FRONTEND noninteractive
ENV TERM xterm
COPY --from=toolchain /src/riscv/bin /usr/bin
COPY --from=toolchain /src/qemu_bin/bin /usr/bin
RUN apt update -y && \
    apt install -y make wget python3-pip && \
    apt clean
RUN rustup override set nightly && \
    rustup target add riscv64gc-unknown-none-elf && \
    cargo install cargo-binutils && \
    rustup +nightly component add rust-src && \
    rustup component add llvm-tools-preview && \
    rustup component add clippy
RUN pip3 install pyfiglet pycryptodome

# ---------------------------------------------------------------------------- #
#                       Full rust image for all functions                      #
# ---------------------------------------------------------------------------- #

# Split more than it should to prevent Pi crashes
FROM rust:bullseye AS rust-full
ENV DEBIAN_FRONTEND noninteractive
ENV TERM xterm
COPY --from=toolchain /src/riscv/bin /usr/bin
COPY --from=toolchain /src/qemu_bin/bin /usr/bin
RUN apt update -y && \
    apt install -y qemu-system make wget graphviz python3-pip && \
    apt clean
RUN rustup override set nightly && \
    rustup target add riscv64gc-unknown-none-elf && \
    cargo install cargo-binutils && \
    rustup +nightly component add rust-src && \
    rustup component add llvm-tools-preview && \
    rustup component add clippy
RUN cargo install cargo-call-stack
RUN cargo install cargo-geiger
RUN cargo install hyperfine
RUN cargo install cargo-expand
RUN pip3 install pyfiglet pycryptodome
