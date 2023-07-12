#!/bin/bash

brctl addbr br0
ip addr flush dev eth1
brctl addif br0 eth1
tunctl -t tap0 -u `whoami`
brctl addif br0 tap0
ifconfig eth1 up
ifconfig tap0 up
ifconfig br0 up
ifconfig br0 192.168.0.1
