section .multiboot_header
    align 8

header_start:
    dd 0xe85250d6       ; magic Multiboot 2 number
    dd 0                ; arch = i386 protected mode
    dd header_end - header_start ; header length
    ; checksum
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

    ; multiboot tags

information_request_tag_start:
        dw 1       ; type
        dw 0       ; flags
        dd information_request_tag_end - information_request_tag_start ; size (12)
        dd 8       ; request framebuffer
information_request_tag_end:
        dd 0       ; for 8-byte alignment

framebuffer_tag_start:
        dw 5       ; type
        dw 0       ; flags
        dd framebuffer_tag_end - framebuffer_tag_start ; size (20)
        dd 1920    ; requested width
        dd 1080    ; requested height
        dd 32      ; color depth (bits per pixel or bpp)
framebuffer_tag_end:
    dd 0           ; for 8-byte alignment

    ; required end tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
header_end: