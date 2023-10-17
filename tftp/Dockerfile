# NEEDS POWERFUL COMPUTER

FROM debian:bullseye-slim

WORKDIR /src

ENV DEBIAN_FRONTEND noninteractive

RUN apt update -y && \
    apt install -y git fakeroot build-essential ncurses-dev \
        xz-utils libssl-dev bc flex libelf-dev bison file \
        wget cpio unzip rsync && \
    apt clean

RUN git clone https://github.com/buildroot/buildroot.git && \
    cd buildroot && git checkout 2023.08

WORKDIR /src/buildroot
RUN make oldconfig

ADD .defconfig /src/buildroot/.defconfig
ADD .config /src/buildroot/.config
RUN make -j$(nproc)
