#!/bin/bash

if [[ $# -ne 1 ]]; then
    echo 'Too many/few arguments, expecting one argument being the path to the loopback device to unmount and detach.' >&2
    exit 1
fi

umount -l $1
losetup --detach $1
