all: debug

export IMG_FILE
export MOUNT_TARGET=/mnt

debug:
	rm -rf ./obj
	mkdir obj

	cargo build
	cp target/x86_64-unknown-dogos-dog/debug/libkernel.a obj

	ld -n -T src/boot/linker.ld -o bin/kernel.elf obj/libkernel.a

release:
	rm -rf ./obj
	mkdir obj

	cargo build -r
	cp target/x86_64-unknown-dogos-dog/release/libkernel.a obj

	ld -n -T src/boot/linker.ld -o bin/kernel.elf obj/libkernel.a

install:
	$(eval $@_LOOPBACK_DEV= $(shell losetup --find --show $$IMG_FILE))
	mount $($@_LOOPBACK_DEV) $$MOUNT_TARGET
	cp bin/kernel.elf $$MOUNT_TARGET/kernel.elf
	../run/umount.sh $($@_LOOPBACK_DEV)

fast-install:
	cp bin/kernel.elf $$MOUNT_TARGET/kernel.elf