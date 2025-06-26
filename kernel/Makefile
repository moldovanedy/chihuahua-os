all: build

export IMG_FILE
export MOUNT_TARGET=/mnt
export RELEASE := true

build:
	rm -rf ./obj
	mkdir obj

ifeq ($(RELEASE), true)
	cargo build -r
	cp target/x86_64-unknown-chihuahuaos-none/release/libkernel.a obj
else
	cargo build
	cp target/x86_64-unknown-chihuahuaos-none/debug/libkernel.a obj
endif

	#create embedded resources
	objcopy -O elf64-x86-64 -I binary res/Tamsyn8x16r.psf obj/Tamsyn8x16r.psf.o

	ld -n -T src/boot/linker.ld -o bin/kernel.elf obj/libkernel.a obj/Tamsyn8x16r.psf.o

clean:
	rm -rf ./bin
	rm -rf ./obj
	rm -rf ./target

install:
	$(eval $@_LOOPBACK_DEV= $(shell losetup --find --show $$IMG_FILE))
	mount $($@_LOOPBACK_DEV) $$MOUNT_TARGET

	mkdir -p $$MOUNT_TARGET/boot
	cp bin/kernel.elf $$MOUNT_TARGET/boot/kernel.elf

	../run/umount.sh $($@_LOOPBACK_DEV)

fast-install:
	mkdir -p $$MOUNT_TARGET/boot
	cp bin/kernel.elf $$MOUNT_TARGET/boot/kernel.elf