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

rm -f /srv/tftp/*
cp .gdbinit /home/l/.gdbinit
cp ./{Image_signed.gz,rootfs.cpio.gz,qemu.dtb,public_key.pem} /srv/tftp/
cp ../bootloader /srv/tftp/
(cd /srv/tftp && gzip --decompress Image_signed.gz)
(cd /srv/tftp && gzip --decompress rootfs.cpio.gz)
# (cd /srv/tftp && mkimage -A riscv -T ramdisk -d rootfs.cpio.gz initrd.img)

printf -v QEMU_CMDLINE '%s' 'qemu-system-riscv64 -M virt ' \
	'-cpu rv64 -smp 1 -m 512 -nographic ' \
	'-display none -serial pipe:/tmp/guest -s ' \
	'-netdev tap,id=mynet0,ifname=tap0,script=no,downscript=no ' \
	'-device e1000,netdev=mynet0,mac=52:55:00:d1:55:01 ' \
	'-kernel u-boot.bin'

wait_for_line() {
	local expected_line_pattern="$1"
	local fifo="$2"
	while read line || [ -n "$line" ]; do
		echo "$line"
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
printf "setenv serverip 10.8.8.1; setenv ipaddr 10.8.8.2; setenv netmask 255.255.255.0; setenv devicetree_image qemu.dtb\n" >/tmp/guest.in

wait_for_line "=>" /tmp/guest.out
echo "✅ Got input prompt"
printf "ping 10.8.8.1\n" >/tmp/guest.in

wait_for_line "is alive" /tmp/guest.out
echo "✅ TFTP Alive"
printf "tftp 0x80100000 \${serverip}:bootloader\n" >/tmp/guest.in

wait_for_line "Bytes transferred" /tmp/guest.out
echo "✅ Kernel transferred"
printf "tftp 0x801fff00 \${serverip}:Image_signed\n" >/tmp/guest.in

wait_for_line "Bytes transferred" /tmp/guest.out
echo "✅ Kernel transferred"
printf "tftp 0x84a00000 \${serverip}:qemu.dtb\n" >/tmp/guest.in

wait_for_line "Bytes transferred" /tmp/guest.out
echo "✅ DTB transferred"
printf "tftp 0x85000000 \${serverip}:rootfs.cpio\n" >/tmp/guest.in
# printf "tftp 0x85000000 \${serverip}:initrd.img\n" >/tmp/guest.in

wait_for_line "Bytes transferred" /tmp/guest.out
echo "✅ RAM disk transferred"
# printf "booti 0x80200100 0x84a00000\n" >/tmp/guest.in
printf "go 0x80100000\n" >/tmp/guest.in

# wait_for_line "Bytes transferred" /tmp/guest.out
# echo "✅ RAM disk transferred"
# printf "booti 0x80200000 0x85000000 0x84a00000\n" > /tmp/guest.in

# sleep 10
# printf "root\n" > /tmp/guest.in
# sleep 2
# printf "root\n" > /tmp/guest.in
# sleep 2
# printf "cat /proc/config.gz | gunzip > running.config\n" > /tmp/guest.in
# sleep 5
# printf "cat running.config\n" > /tmp/guest.in

wait_for_line "Welcome" /tmp/guest.out

rm -f /tmp/{guest,host}.{in,out}
kill -9 $pid
python3 kill_hanging_qemu.py
exit
