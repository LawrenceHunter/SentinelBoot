#!/bin/bash

scp -O bootloader.img tftp/* root@$1:/tftpboot/boot
scp -O tftp/.gdbinit root@$1:/root
