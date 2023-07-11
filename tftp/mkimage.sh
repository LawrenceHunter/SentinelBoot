#!/bin/bash
mkimage -A riscv -T script -O efi -C none -d boot.scr boot.scr.uimg
