OUTPUT_FORMAT(elf64-x86-64)
ENTRY(kmain)

SECTIONS {
    /* load at virtual address of 2GiB (higher-half kernel) */
    . = 0xffffffff80000000;

    .text BLOCK(4K) : ALIGN(4K)
	{
		*(.text)
	}

    /* Read-only data. */
	.rodata BLOCK(4K) : ALIGN(4K)
	{
		*(.rodata)
	}

	/* Read-write data (initialized) */
	.data BLOCK(4K) : ALIGN(4K)
	{
		*(.data)
	}

    .bss BLOCK(4K) : ALIGN(4K)
	{
		*(COMMON)
		*(.bss)
	}
}
