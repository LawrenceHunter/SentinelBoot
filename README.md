
# SentinelBoot - A demonstrative secure bootloader

This repository is the basis for a final year project at The University of Manchester. SentinelBoot is a demonstrative project to improve memory safety through both safe principles and the Rust programming language; additionally, SentinelBoot uses public-key cryptography to verify the kernel's hash before booting.

SentinelBoot also supports the use of the [RISC-V vector cryptography extension](https://github.com/riscv/riscv-crypto).


## Build

To build locally the use of the Docker container is recommended and will automatically be invoked by the Makefile. The default target is `qemu` which is built for running under QEMU without the use of global memory allocation or vector cryptography.

- The target can be changed by specifying `BSP=<target>`
- Debug printing can be enabled by `DEBUG=y`
- Using Docker for building and running can be disabled by `DOCKER=n`
- Clearing the terminal on build invocation can be disabled by `CLEAR=n`

| Target        | Board                  | Features                 |
|---------------|------------------------|--------------------------|
| `qemu`        | QEMU                   |                          |
| `qemu_vector` | QEMU                   | Vector Cryptography      |
| `qemu_alloc`  | QEMU                   | Global memory allocation |
| `unmatched`   | HiFive Unmatched Rev B |                          |
| `visionfive`  | StarFive VisionFive 2  |                          |

## Setup

To run SentinelBoot involves some setup due to the need for a tftp server; however, setup may include the following. After which QEMU will be able to pull the SentinelBoot binary, keys, linux binary, dtb, and rootfs from the server.

### Setup dhcpd
```bash
apt install isc-dhcp-server -y
echo "auto eth1
iface eth1 inet static
        address 192.168.0.1
        netmask 255.255.255.0" > /etc/network/interfaces
echo "INTERFACESv4=\"eth1\"" > /etc/defaults/isc-dhcp-server
echo "subnet 192.168.0.0 netmask 255.255.255.0 {
        interface br0;
        authorative;
        host vision-five {
                hardware ethernet 6c:cf:39:00:1d:11;
                fixed-address 192.168.0.2;
                next-server 192.168.0.1;
                filename Image;
        }
        host qemu {
                hardware ethernet 52:55:00:d1:55:01;
                fixed-address 192.168.0.3;
                next-server 192.168.0.1;
                filename Image;
        }
}" > /etc/dhcp/dhcpd.conf
ifconfig eth1 up
ifconfig eth1 192.168.0.1
systemctl restart isc-dhcp-server
```

### Setup tftp
```bash
apt install tftpd-hpa -y
echo "TFTP_USERNAME=\"tftp\"
TFTP_DIRECTORY=\"/tftpboot/boot\"
TFTP_ADDRESS=\"192.168.0.1:69\"
TFTP_OPTIONS=\"--ipv4 --secure --create\"" > /etc/default/tftpd-hpa
mkdir -p /tftpboot/boot
chown tftp:tftp /tftpboot/boot
/etc/init.d/tftpd-hpa restart
```

### mkimage
```bash
apt install u-boot-tools -y
```

### Setup minicom
```bash
apt install minicom -y
```

### Setup network bridge
```bash
brctl addbr br0
ip addr flush dev eth1
brctl addif br0 eth1
tunctl -t tap0 -u `whoami`
brctl addif br0 tap0
ifconfig eth1 up
ifconfig tap0 up
ifconfig br0 up
ifconfig br0 192.168.0.1
systemctl restart isc-dhcp-server
/etc/init.d/tftpd-hpa restart
```
## Run

To run SentinelBoot we need to specify a known MAC address to QEMU such that the DHCP is able to recognise it and give it a fixed ip address. After which we need to request the binaries from the server before jumping to the SentinelBoot. It is important to note here U-boot handles the initial work of handling tftp this is a deliberate decision to reduce the scope of the project and prevent too much time being spent on driver development.

### Run QEMU
```bash
qemu-system-riscv64 -M virt \
	-cpu rv64,v=true,vlen=1024,rvv_ma_all_1s=true,\
    rvv_ta_all_1s=true,zvbb=true,zvbc=true,zvknha=true \
	-smp 1 -m 512 -nographic \
	-display none -serial pipe:/tmp/guest -s \
	-netdev tap,id=mynet0,ifname=tap0,script=no,downscript=no \
	-device e1000,netdev=mynet0,mac=52:55:00:d1:55:01 \
	-kernel ./tftp/u-boot.bin
```

### U-Boot step
```bash
=> setenv serverip 10.8.8.1; setenv ipaddr 10.8.8.2
=> setenv netmask 255.255.255.0
=> setenv devicetree_image qemu.dtb
=> tftp 0x80100000 ${serverip}:sentinel_boot
...
=> tftp 0x801fff00 ${serverip}:Image_signed
...
=> tftp 0x84a00000 ${serverip}:qemu.dtb
...
=> tftp 0x85000000 ${serverip}:rootfs.cpio
...
=> go 0x80100000
...
```
After `go 0x80100000` execution is handed to SentinelBoot and will continue as expected.
## Documentation

As this is a Rust project we can make use of the built in documentation handling as such the docs can be built by `make doc`.


## Acknowledgements

 - [Dr Pierre Olivier](https://research.manchester.ac.uk/en/persons/pierre.olivier) - Supervisor
 - [Codethink](https://www.codethink.co.uk) - Industrial sponsor

## Authors

- [@lawrencehunter](https://www.github.com/lawrencehunter) - sole


## Contributing & Feedback

Contributions and feedback are always welcome; however, after the deadline this will not be actively worked on so response times may be slow.
