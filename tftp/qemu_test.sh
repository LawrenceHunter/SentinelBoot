#!/bin/bash

if [ "$EUID" -ne 0 ]; then
	echo "Please run as root"
	exit
fi

set -e          # Abort script at first error, when a command exits with non-zero status
set -u          # Using an undefined variable outputs error message, and forces exit
set -o pipefail # CReturn the exit of the last command in the pipe
set -x          # Similar to verbose mode (-v), but expands commands

trap ctrl_c INT

function ctrl_c() {
	kill -9 $pid
}

rm -f /tmp/{guest,host}.{in,out} && mkfifo /tmp/{guest,host}.{in,out}
set +x

rm -f /tftpboot/boot/*
cp ./tftp/{Image_signed.gz,rootfs.cpio.gz,qemu.dtb} /tftpboot/boot
cp ./sentinel_boot /tftpboot/boot
(cd /tftpboot/boot && gzip --decompress Image_signed.gz)
(cd /tftpboot/boot && gzip --decompress rootfs.cpio.gz)

printf -v QEMU_CMDLINE '%s' 'qemu-system-riscv64 -M virt ' \
	'-cpu rv64,v=true,vlen=1024,rvv_ma_all_1s=true,rvv_ta_all_1s=true,zvbb=true,zvbc=true,zvknha=true '\
	'-smp 1 -m 512 -nographic ' \
	'-display none -serial pipe:/tmp/guest -s ' \
	'-netdev tap,id=mynet0,ifname=tap0,script=no,downscript=no ' \
	'-device e1000,netdev=mynet0,mac=52:55:00:d1:55:01 ' \
	'-kernel ./tftp/u-boot.bin'

wait_for_line() {
	local expected_line_pattern="$1"
	local fifo="$2"
	while read line || [ -n "$line" ]; do
		echo "  [$(date +"%T")] $line"
		if [[ $line == *$expected_line_pattern* ]]; then
			break
		fi
	done <$fifo
}

echo "❕ Running QEMU in the background..."
eval "$QEMU_CMDLINE" &
pid=$!

wait_for_line "eth0" /tmp/guest.out
printf "a\n" >/tmp/guest.in
wait_for_line "=>" /tmp/guest.out
printf "setenv serverip 192.168.0.1; setenv ipaddr 192.168.0.3; setenv netmask 255.255.255.0; setenv devicetree_image qemu.dtb\n" >/tmp/guest.in

wait_for_line "=>" /tmp/guest.out
echo "✅ Got input prompt"
printf "tftp 0x80100000 \${serverip}:sentinel_boot\n" >/tmp/guest.in

wait_for_line "Bytes transferred" /tmp/guest.out
echo "✅ Kernel transferred"
printf "tftp 0x801fff00 \${serverip}:Image_signed\n" >/tmp/guest.in

wait_for_line "Bytes transferred" /tmp/guest.out
echo "✅ Kernel transferred"
printf "tftp 0x84a00000 \${serverip}:qemu.dtb\n" >/tmp/guest.in

wait_for_line "Bytes transferred" /tmp/guest.out
echo "✅ DTB transferred"
printf "tftp 0x85000000 \${serverip}:rootfs.cpio\n" >/tmp/guest.in

wait_for_line "Bytes transferred" /tmp/guest.out
echo "✅ RAM disk transferred"
printf "go 0x80100000\n" >/tmp/guest.in

wait_for_line "Welcome" /tmp/guest.out

rm -f /tmp/{guest,host}.{in,out}
kill -9 $pid
exit
