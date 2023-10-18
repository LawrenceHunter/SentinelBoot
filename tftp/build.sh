#!/bin/zsh
# NEEDS POWERFUL COMPUTER

docker build --tag buildroot --file Dockerfile . && \
docker run -it buildroot:latest &&
docker cp $(docker create --name tc buildroot:latest):/src/buildroot/output/images/Image ./ &&
docker cp $(docker create --name tc buildroot:latest):/src/buildroot/output/images/rootfs.ext4 ./ &&
GZIP=-9 tar cvzf Image.tar.gz Image && rm Image && \
GZIP=-9 tar cvzf rootfs.ext4.tar.gz rootfs.ext4 && rm rootfs.ext4
