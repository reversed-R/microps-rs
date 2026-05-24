#!/bin/bash

sudo ip tuntap add mode tap user $USER name tap0
sudo ip addr add 192.0.2.1/24 dev tap0
sudo sysctl -w net.ipv6.conf.tap0.disable_ipv6=1
sudo ip link set tap0 up
