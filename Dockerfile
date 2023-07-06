FROM python:3.9
ENV DEBIAN_FRONTEND noninteractive
ENV TERM xterm
ENV PATH="/usr/bin:/root/.cargo/bin:${PATH}"

RUN apt update -y && \
    apt install -y qemu-system make wget autoconf gcc genext2fs libconfuse-dev git \
    pkg-config file g++ cpio unzip rsync bc build-essential libc6 libc-bin locales \
    curl libncurses5-dev libncursesw5-dev flex bison libssl-dev graphviz libyaml-dev \
    libgmp3-dev libmpc-dev gawk libsigsegv2 libpython3-dev software-properties-common \
    kmod automake autotools-dev texinfo xxd gdisk gperf libmpfr-dev libtool patchutils \
    python3 screen zlib1g-dev bc dosfstools mtools device-tree-compiler libglib2.0-dev \
    libpixman-1-dev kpartx && \
    # add-apt-repository ppa:deadsnakes/ppa -y && \
    wget https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download/2023.05.27/riscv64-glibc-ubuntu-20.04-nightly-2023.05.27-nightly.tar.gz && \
    tar -xvf riscv64-glibc-ubuntu-20.04-nightly-2023.05.27-nightly.tar.gz && \
    rm riscv64-glibc-ubuntu-20.04-nightly-2023.05.27-nightly.tar.gz && \
    cp riscv/bin/* /usr/bin && \
    rm -rf riscv && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    rustup override set nightly && \
    rustup target add riscv64gc-unknown-none-elf && \
    cargo install cargo-binutils && \
    cargo install cargo-call-stack && \
    cargo install cargo-geiger && \
    rustup +nightly component add rust-src && \
    rustup component add llvm-tools-preview && \
    rustup component add clippy && \
    mkdir genimage && cd genimage && \
    git clone https://github.com/pengutronix/genimage.git . && \
    ./autogen.sh && \
    ./configure && \
    make -j$(nproc) && \
    cp ./genimage /usr/bin && \
    cd .. && rm -rf genimage && \
    apt clean
