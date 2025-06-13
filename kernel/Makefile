all: build

build:
	rm -rf ./obj
	mkdir obj

	cargo build
	cp target/x86_64-unknown-dogos-dog/debug/libkernel.a obj

	nasm -f elf64 src/boot/multiboot_header.asm -o obj/multiboot_header.o
	nasm -f elf64 src/boot/init.asm -o obj/init.o
	nasm -f elf64 src/boot/boot_long.asm -o obj/boot_long.o
	ld -n -T src/boot/linker.ld -o bin/kernel.elf \
	  obj/multiboot_header.o \
	  obj/init.o \
	  obj/boot_long.o \
	  obj/libkernel.a

release:
	rm -rf ./obj
	mkdir obj

	cargo build -r
	cp target/x86_64-unknown-dogos-dog/release/libkernel.a obj

	nasm -f elf64 src/boot/multiboot_header.asm -o obj/multiboot_header.o
	nasm -f elf64 src/boot/init.asm -o obj/init.o
	nasm -f elf64 src/boot/boot_long.asm -o obj/boot_long.o
	ld -n -T src/boot/linker.ld -o bin/kernel.elf \
	  obj/multiboot_header.o \
	  obj/init.o \
	  obj/boot_long.o \
	  obj/libkernel.a

grub-boot:
	grub2-mkstandalone -O x86_64-efi -o bin/BOOTX64.EFI "boot/grub/grub.cfg=src/boot/grub.cfg"