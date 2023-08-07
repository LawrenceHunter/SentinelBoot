#!/bin/bash

set -e # Abort script at first error, when a command exits with non-zero status
set -u # Using an undefined variable outputs error message, and forces exit
set -o pipefail # CReturn the exit of the last command in the pipe
set -x # Similar to verbose mode (-v), but expands commands

rm -f /tmp/{guest,host}.{in,out} && mkfifo /tmp/{guest,host}.{in,out}
set +x

printf -v QEMU_CMDLINE '%s' 'qemu-system-riscv64 -serial pipe:/tmp/guest ' \
'-M virt -cpu rv64 -smp 4 -m 128M -nographic -bios none -kernel bootloader.img'

wait_for_line () {
    local expected_line_pattern="$1"
    local fifo="$2"
    while read -r line; do
        echo "  [$(date +"%T")] $line"
        if [[ $line == *$expected_line_pattern* ]]; then
            break
        fi
    done < $fifo
}

echo "❕ Running QEMU in the background..."
eval "$QEMU_CMDLINE" &

echo "❕ Waiting for 'bootloader version'..."
wait_for_line "bootloader version" /tmp/guest.out
echo "✅ Got 'bootloader version'"

echo "❕ Waiting for 'Echoing input now'..."
wait_for_line "Echoing input now" /tmp/guest.out
echo "✅ Got 'Echoing input now'"

echo "✅ All expected output achieved!"
rm -f /tmp/{guest,host}.{in,out}
exit
