all: build-all

export IMG_FILE
export MOUNT_TARGET=/mnt

build-all:
	$(MAKE) -C bootloader
	$(MAKE) -C kernel

clean:
	$(MAKE) -f bootloader/Makefile clean
	$(MAKE) -f kernel/Makefile clean

install-all:
	$(eval $@_LOOPBACK_DEV= $(shell losetup --find --show $$IMG_FILE))
	mount $($@_LOOPBACK_DEV) $$MOUNT_TARGET

	$(MAKE) -f bootloader/Makefile fast-install
	$(MAKE) -f kernel/Makefile fast-install

	../run/umount.sh $($@_LOOPBACK_DEV)