#!/bin/bash

if [ "$EUID" -ne 0 ]
  then echo "Please run as root"
  exit
fi

set -e # Abort script at first error, when a command exits with non-zero status
set -u # Using an undefined variable outputs error message, and forces exit
set -o pipefail # CReturn the exit of the last command in the pipe
set -x # Similar to verbose mode (-v), but expands commands

trap ctrl_c INT

function ctrl_c() {
    kill -9 $pid
}

rm -f /tmp/{guest,host}.{in,out} && mkfifo /tmp/{guest,host}.{in,out}
set +x

cp ./* /srv/tftp/
cp ../bootloader /srv/tftp/
(cd /srv/tftp && tar xvf Image.tar.gz)

printf -v QEMU_CMDLINE '%s' 'qemu-system-riscv64 -M virt ' \
    '-cpu rv64 -smp 2 -m 512 -nographic ' \
    '-display none -serial pipe:/tmp/guest -s ' \
    '-netdev tap,id=mynet0,ifname=tap0,script=no,downscript=no ' \
    '-device e1000,netdev=mynet0,mac=52:55:00:d1:55:01 ' \
    '-bios u-boot.bin'

wait_for_line () {
    local expected_line_pattern="$1"
    local fifo="$2"
    while read line; do
        echo "  [$(date +"%T")] $line"
        if [[ $line == *$expected_line_pattern* ]]; then
            break
        fi
    done < $fifo
}

echo "❕ Running QEMU in the background..."
eval "$QEMU_CMDLINE" &
pid=$!

echo "❕ Waiting for 'OpenThesis version'..."
wait_for_line "serverip" /tmp/guest.out
wait_for_line "serverip" /tmp/guest.out
printf " setenv serverip 10.8.8.1; setenv ipaddr 10.8.8.2; setenv netmask 255.255.255.0;\n" > /tmp/guest.in
echo "✅ Got input prompt"

echo "❕ Waiting for 'Drivers loaded'..."
wait_for_line "=>" /tmp/guest.out
printf "ping 10.8.8.1\n" > /tmp/guest.in

wait_for_line "is alive" /tmp/guest.out
echo "✅ TFTP Alive"

printf "tftpboot 0x80000000 bootloader\n" > /tmp/guest.in

wait_for_line "Bytes transferred" /tmp/guest.out
printf "tftpboot 0x80100000 Image\n" > /tmp/guest.in

wait_for_line "Bytes transferred" /tmp/guest.out
printf "go 0x80000000\n" > /tmp/guest.in

wait_for_line "Bytes transferred" /tmp/guest.out
printf "md 0x80000000 0x100\n" > /tmp/guest.in

wait_for_line "OpenThesis version" /tmp/guest.out
echo "✅ Got 'OpenThesis version'"
wait_for_line "EXECUTION DONE" /tmp/guest.out

rm -f /tmp/{guest,host}.{in,out}
kill -9 $pid
exit
