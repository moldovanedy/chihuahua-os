ENTRY(start)

SECTIONS {
    . = 2M;

    .text BLOCK(4K) : ALIGN(4K)
	{
		*(.multiboot_header)
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
