#!/bin/zsh
# NEEDS POWERFUL COMPUTER

docker build --tag buildroot --file Dockerfile.buildroot . && \
# docker run -it buildroot:latest && \
docker rm tc; \
docker cp $(docker create --name tc buildroot:latest):/src/buildroot/output/images/Image.gz ./ &&
docker rm tc && \
docker cp $(docker create --name tc buildroot:latest):/src/buildroot/output/images/rootfs.cpio.gz ./ &&
docker rm tc
