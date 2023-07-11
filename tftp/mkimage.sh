#!/bin/bash
mkimage -A riscv -T script -O linux -C none -d boot.scr boot.scr.uimg
mkimage -A riscv -T kernel -O linux -C none -d bootloader.img -a 0x80000000 -e 0x80000000 Image
