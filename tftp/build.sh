#!/bin/zsh
# NEEDS POWERFUL COMPUTER

docker build --tag buildroot --file Dockerfile.buildroot . && \
# docker run -it buildroot:latest && \
docker rm tc; \
docker cp $(docker create --name tc buildroot:latest):/src/buildroot/output/images/Image.gz ./ &&
docker rm tc && \
docker cp $(docker create --name tc buildroot:latest):/src/buildroot/output/images/rootfs.cpio.gz ./ &&
docker rm tc && \
GZIP=-9 tar cvzf Image.gz.tar.gz Image.gz && rm Image.gz && \
GZIP=-9 tar cvzf rootfs.cpio.gz.tar.gz rootfs.cpio.gz && rm rootfs.cpio.gz
