global kernel_start
extern kmain ; kernel main function
extern mb2_magic_number
extern mb2_info

section .text
    bits 64
kernel_start:
    ; load 0 into all data segment registers
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; load the previous values and extend them to 64 bits
    mov eax, dword [mb2_magic_number]
    mov rax, rax

    mov ebx, dword [mb2_info]
    mov rbx, rbx

    ; load the necessary values
    mov rdi, rax
    mov rsi, rbx
    call kmain