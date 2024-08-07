#!/bin/bash

# Copyright (c) 2023-2024 Lawrence Hunter <lawrence.hunter@outlook.com

if [ "$EUID" -ne 0 ]
  then echo "Please run as root"
  exit
fi

set -x

ETH=ens160   #Ethernet interface in your HOST

if [ $# -eq 1 ]
  then
    ETH=ens160
else
    ETH=$2
fi


USR=l    #User name- whoami

alias ifup_tap='sudo ~/ProjectWork/bin/tuntap.sh up'
alias ifdown_tap='sudo ~/ProjectWork/bin/tuntap.sh down'

if test -z $1 ; then
        echo need a arg: down/up
        exit
fi

if [ "up" = $1 ] ; then

        brctl addbr br0

        ifconfig $ETH down
        brctl addif br0 $ETH

        #Close Spanning Tree Protocol
        brctl stp br0 off

        ifconfig br0 10.8.8.1 netmask 255.255.255.0 promisc up  #3
        ifconfig $ETH 10.8.8.10 netmask 255.255.255.0 promisc up #2

        tunctl -t tap0 -u $USR
        ifconfig tap0 10.8.8.11 netmask 255.255.255.0 promisc up #4
        brctl addif br0 tap0
else
        ifconfig tap0 down
        brctl delif br0 tap0

        ifconfig $ETH down
        brctl delif br0 enp2s0

        ifconfig br0 down
        brctl delbr br0

        ifconfig $ETH 10.8.8.10 netmask 255.255.255.0  #2
fi
