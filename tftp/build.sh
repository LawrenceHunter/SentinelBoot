#!/bin/zsh
# NEEDS POWERFUL COMPUTER

docker build --tag buildroot --file Dockerfile . && \
docker cp $(docker create --name tc buildroot:latest):/src/buildroot/output/images/Image ./ &&
GZIP=-9 tar cvzf Image.tar.gz Image && rm Image
