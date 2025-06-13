global start
global mb2_magic_number
global mb2_info

%define STACK_SIZE 0x4000 ; 16KiB of stack
extern kernel_start ; defined in boot_long.asm

section .bss
    align 4096
; sets up identity paging with 2 MiB huge-pages
p4_table: resb 4096

p3_table: resb 4096

p2_table_0: resb 4096
p2_table_1: resb 4096
p2_table_2: resb 4096
p2_table_3: resb 4096

stack_bottom:
    resb STACK_SIZE ; reserve 16KiB of stack memory
stack_top:

mb2_magic_number: resd 1
mb2_info: resd 1
; END section .bss

section .rodata
gdt64:
    dq 0 ; zero entry
    .code: equ $ - gdt64
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; it's a code segment | type (always 1) | valid sector | use 64-bit

    .pointer:
        dw $ - gdt64 - 1
        dq gdt64
; END section .rodata

section .text
    bits 32
    
start:
    mov esp, stack_top
    mov [mb2_magic_number], eax ; Magic Multiboot2 value
    mov [mb2_info], ebx  ; Multiboot2 info

    call check_long_mode
    call set_up_page_tables
    call enable_paging

    lgdt [gdt64.pointer] ; load GDT
    jmp gdt64.code:kernel_start ; jump to boot_long.asm and start the kernel

    ; the next code should be unreachable (we shouldn't be able to return from kmain), but that case is treated regardless
    cli
inf_loop:
    hlt
	jmp inf_loop
; END start


check_long_mode:
    ; test if extended processor info is available
    mov eax, 0x80000000    ; implicit argument for CPUID
    cpuid                  ; get highest supported argument
    cmp eax, 0x80000001    ; it needs to be at least 0x80000001
    jb .no_long_mode       ; if it's less, the CPU is too old for long mode

    ; use extended info to test if long mode is available
    mov eax, 0x80000001    ; argument for extended processor info
    cpuid                  ; returns various feature bits in ecx and edx
    test edx, 1 << 29      ; test if the LM-bit is set in the D-register
    jz .no_long_mode       ; If it's not set, there is no long mode
    ret

    .no_long_mode:
        mov al, "2"
        ret
; END check_long_mode

set_up_page_tables:
    ; map first P4 entry to P3 table
    mov eax, p3_table
    or eax, 0b11 ; present + writable
    mov [p4_table + 8 * 0], eax

    ; map first P3 entry to first P2 table
    mov eax, p2_table_0
    or eax, 0b11 ; present + writable
    mov [p3_table], eax

    ; ... to second P2 table
    mov eax, p2_table_1
    or eax, 0b11 ; present + writable
    mov [p3_table + 8 * 1], eax

    ; ... to third P2 table
    mov eax, p2_table_2
    or eax, 0b11 ; present + writable
    mov [p3_table + 8 * 2], eax

    ; ... to fourth P2 table
    mov eax, p2_table_3
    or eax, 0b11 ; present + writable
    mov [p3_table + 8 * 3], eax

    ; map each P2 entry to a huge 2MiB page

    mov ecx, 0         ; counter variable
    .map_p2_table_0:
        ; map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx
        mov eax, 0x200000  ; 2MiB
        mul ecx            ; start address of ecx-th page
        or eax, 0b10000011 ; present + writable + huge
        mov [p2_table_0 + ecx * 8], eax ; map ecx-th entry
    
        inc ecx
        cmp ecx, 512       ; if counter == 512, the whole P2 table is mapped
        jne .map_p2_table_0  ; else map the next entry

    mov ecx, 0         ; counter variable
    .map_p2_table_1:
        ; map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx
        mov eax, 0x200000  ; 2MiB
        mul ecx            ; start address of ecx-th page
        or eax, 0b10000011 ; present + writable + huge
        mov [p2_table_1 + ecx * 8], eax ; map ecx-th entry
    
        inc ecx
        cmp ecx, 512       ; if counter == 512, the whole P2 table is mapped
        jne .map_p2_table_1  ; else map the next entry

    mov ecx, 0         ; counter variable
    .map_p2_table_2:
        ; map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx
        mov eax, 0x200000  ; 2MiB
        mul ecx            ; start address of ecx-th page
        or eax, 0b10000011 ; present + writable + huge
        mov [p2_table_2 + ecx * 8], eax ; map ecx-th entry
    
        inc ecx
        cmp ecx, 512       ; if counter == 512, the whole P2 table is mapped
        jne .map_p2_table_2  ; else map the next entry

    mov ecx, 0         ; counter variable
    .map_p2_table_3:
        ; map ecx-th P2 entry to a huge page that starts at address 2MiB*ecx
        mov eax, 0x200000  ; 2MiB
        mul ecx            ; start address of ecx-th page
        or eax, 0b10000011 ; present + writable + huge
        mov [p2_table_3 + ecx * 8], eax ; map ecx-th entry
    
        inc ecx
        cmp ecx, 512       ; if counter == 512, the whole P2 table is mapped
        jne .map_p2_table_3  ; else map the next entry

    ret
; END set_up_page_tables

enable_paging:
    ; load P4 to cr3 register
    mov eax, p4_table
    mov cr3, eax

    ; enable PAE-flag in cr4 (Physical Address Extension)
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; set the long mode bit in the EFER MSR (model specific register)
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret
; END enable_paging
; END section .text