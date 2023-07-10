#!/bin/bash
mkimage -A riscv -T invalid -O invalid -C none -a 0 -e 0 -d sdcard.img uImage
