#!/bin/bash

if [[ $# -ne 1 ]]; then
    echo 'Too many/few arguments, expecting only the path to the .img file to mount' >&2
    exit 1
fi

loopback_dev="$(losetup --find --show $1)"
echo "Loopback device is: $loopback_dev. You need to specify this when unmounting."
mount $loopback_dev /mnt
