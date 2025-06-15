all: debug-all

export IMG_FILE
export MOUNT_TARGET=/mnt

debug-all:
	$(MAKE) -C bootloader
	$(MAKE) -C kernel

install-all:
	$(eval $@_LOOPBACK_DEV= $(shell losetup --find --show $$IMG_FILE))
	mount $($@_LOOPBACK_DEV) $$MOUNT_TARGET

	$(MAKE) -f bootloader/Makefile fast-install

	../run/umount.sh $($@_LOOPBACK_DEV)