all: debug

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