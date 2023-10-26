#!/bin/bash

qemu-system-riscv64 -M virt \
    -cpu rv64 -smp 2 -m 512 -nographic \
    -display none -serial mon:stdio -s \
    -netdev tap,id=mynet0,ifname=tap0,script=no,downscript=no \
    -device e1000,netdev=mynet0,mac=52:55:00:d1:55:01 \
    -kernel u-boot.bin
