all: build-all

export IMG_FILE
export MOUNT_TARGET=/mnt

build-all:
	$(MAKE) -C bootloader
	$(MAKE) -C kernel

clean:
	$(MAKE) -C bootloader clean
	$(MAKE) -C kernel clean

install-all:
	$(eval $@_LOOPBACK_DEV= $(shell losetup --find --show $$IMG_FILE))
	mount $($@_LOOPBACK_DEV) $$MOUNT_TARGET

	$(MAKE) -C bootloader fast-install
	$(MAKE) -C kernel fast-install

	run/umount.sh $($@_LOOPBACK_DEV)