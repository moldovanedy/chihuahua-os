all: debug

export IMG_FILE
export MOUNT_TARGET=/mnt

debug:
	rm -rf ./bin
	mkdir bin

	cargo build --target x86_64-unknown-uefi
	cp target/x86_64-unknown-uefi/debug/bootloader.efi bin
	mv bin/bootloader.efi bin/BOOTX64.EFI

clean:
	rm -rf ./bin
	rm -rf ./target

install:
	$(eval $@_LOOPBACK_DEV= $(shell losetup --find --show $$IMG_FILE))
	mount $($@_LOOPBACK_DEV) $$MOUNT_TARGET

	mkdir -p $$MOUNT_TARGET/EFI/BOOT
	cp bin/BOOTX64.EFI $$MOUNT_TARGET/EFI/BOOT/BOOTX64.EFI

	../run/umount.sh $($@_LOOPBACK_DEV)

fast-install:
	mkdir -p $$MOUNT_TARGET/EFI/BOOT
	cp bin/BOOTX64.EFI $$MOUNT_TARGET/EFI/BOOT/BOOTX64.EFI