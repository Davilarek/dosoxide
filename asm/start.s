.intel_syntax noprefix
.macro DEBUG_CHAR char, pos
    push ax
    push es
    mov ax, 0xB800
    mov es, ax
    mov byte ptr es:[\pos], \char
    mov byte ptr es:[\pos + 1], 0x4F
    pop es
    pop ax
.endm
.section .entry16, "ax"
.code16
.global _start
_start:
    cli
    cld
    mov ax, cs
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0xFFF0
    movzx eax, ax
    shl eax, 4
    mov ebx, eax
    .irp desc, gdt_code32, gdt_data32, gdt_code16, gdt_data16
        mov eax, ebx
        mov word ptr [\desc + 2], ax
        shr eax, 16
        mov byte ptr [\desc + 4], al
        mov byte ptr [\desc + 7], ah
    .endr
    mov word ptr [thunk_ptr], offset static_thunk
    mov [thunk_ptr + 2], cs
    mov eax, ebx
    add eax, offset gdt_start
    mov [gdt_ptr + 2], eax
    mov [rm_cs], cs
    mov [rm_ss_val], ss
    mov [rm_sp_val], sp
    lgdt [gdt_ptr]
    // lidt [idt_ptr]
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    .byte 0x66, 0xEA
    .long protected_mode_entry
    .word 0x08
.align 4
static_thunk:
    .byte 0x90, 0x90
    .byte 0xC3
.align 8
idt_start: .fill 2048, 1, 0
idt_ptr:   .word 2047; .long 0
gdt_start:
    .quad 0x0000000000000000
gdt_code32:
    .quad 0x00CF9A000000FFFF
gdt_data32:
    .quad 0x00CF92000000FFFF
gdt_code16:
    // .quad 0x00009A000000FFFF
    .quad 0x008F9A000000FFFF
// gdt_data16:
//     .quad 0x000092000000FFFF
gdt_data16:
    .quad 0x008F92000000FFFF
gdt_phys:
    .quad 0x00CF92000000FFFF
gdt_end:
gdt_ptr: .word gdt_end - gdt_start - 1; .long 0
stack_16bit:
    // .fill 256, 1, 0
    .fill 2048, 1, 0
stack_16bit_top:
.section .data
saved_esp: .long 0
real_jmp:
    .byte 0xEA
real_jmp_offset:
    .word 0
real_jmp_seg:
    .word 0
real_mode_ptr:
    .word 0
    .word 0
.align 4
pm16_seg: .word 0
pm16_off: .word 0
.align 4
rm_regs_ptr: .long 0
rm_int_no: .byte 0

rm_cs:     .word 0
rm_ss_val: .word 0
rm_sp_val: .word 0
SAVED_CR3: .long 0
.align 2
thunk_ptr:
    .word 0
    .word 0
rm_rm_cs:  .word 0
rm_rm_ip:  .word 0
.align 2
rm_far_jump:
    .byte 0xEA
    .word 0x0000
    .word 0x0000
.section .text32, "ax"
.code32
protected_mode_entry:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    // mov esp, 0xFFF0
    // mov esp, 0x70000
    // mov esp, 0x80000
    mov esp, 0x7C0000
    mov ax, 0x28
    mov fs, ax
    mov byte ptr fs:[0xB800C], '7'
    mov byte ptr fs:[0xB800D], 0x0A
    push ebx
    call rust_main
.section .text32, "ax"
.code32
.global call_rm_int
call_rm_int:
    push ebp
    mov ebp, esp
    pushad
    cli
    mov eax, cr0
    test eax, 0x80000000
    jz .no_pg
    mov edx, cr3
    mov [SAVED_CR3], edx
    and eax, 0x7FFFFFFF
    mov cr0, eax
    jmp 1f
1:
.no_pg:
    movzx eax, byte ptr [ebp + 8]
    mov [rm_int_no], al
    mov byte ptr fs:[0x500], al
    mov edi, [ebp + 12]

    movzx eax, word ptr [rm_cs]
    shl eax, 4
    add edi, eax

    mov esi, esp
    mov [saved_esp], esi
    push 0x18
    push offset to_16bit_pm
    .byte 0x66, 0xCB
.section .entry16, "ax"
.align 16
.code16
to_16bit_pm:
    mov ax, 0x28
    mov es, ax
    mov ebx, 0xB801C
    mov byte ptr es:[ebx], '5'
    mov byte ptr es:[ebx+1], 0x0C
    mov ax, 0x20
    mov ds, ax
    mov ss, ax
    // mov sp, offset stack_16bit + 256
    mov sp, offset stack_16bit_top
    movzx ebx, word ptr [rm_cs]
    shl ebx, 4
    add ebx, offset to_real_mode
    mov ecx, ebx
    shr ecx, 4
    mov edx, ebx
    and edx, 0xF
    mov ebp, cr0
    and ebp, 0xFFFFFFFE
    mov cr0, ebp
    mov word ptr [real_mode_ptr], dx
    mov word ptr [real_mode_ptr+2], cx
    .att_syntax
    ljmp *real_mode_ptr
    .intel_syntax noprefix
.align 16
.code16
.global to_real_mode
to_real_mode:
    DEBUG_CHAR 'A', 0x1E
    mov ax, cs
    mov ds, ax
    mov es, ax
    mov bx, [rm_ss_val]
    mov cx, [rm_sp_val]
    DEBUG_CHAR 'B', 0x20
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, bx
    mov sp, cx
    DEBUG_CHAR 'C', 0x22

    mov al, [0x500]
    mov byte ptr [0x600], 0xCD
    mov byte ptr [0x601], al
    mov byte ptr [0x602], 0xCB

    push edi
    mov eax, [edi + 0]
    mov ebx, [edi + 4]
    mov ecx, [edi + 8]
    mov edx, [edi + 12]
    mov esi, [edi + 16]
    pop ebp

    sti
    .att_syntax
    lcall $0x0000, $0x600
    .intel_syntax noprefix
    cli

    pushf
    pop ax
    mov [ebp + 0], eax
    mov [ebp + 12], edx
    mov [ebp + 24], ax
    DEBUG_CHAR 'F', 0x28
    mov ax, cs
    movzx ebx, ax
    shl ebx, 4
    .att_syntax
    subl $to_real_mode_masked, %ebx
    .intel_syntax noprefix
    add ebx, offset gdt_start
    mov ax, offset gdt_end
    sub ax, offset gdt_start
    dec ax
    push ebx
    push ax
    mov bp, sp
    lgdt [bp]
    add sp, 6
    DEBUG_CHAR 'G', 0x2A
    mov eax, cr0
    or al, 1
    mov cr0, eax
    .byte 0x66, 0xEA
    .long to_32bit_pm_back
    .word 0x08
.section .text32
.code32

to_32bit_pm_back:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov esp, [saved_esp]
    mov eax, [SAVED_CR3]
    test eax, eax
    jz .no_paging
    mov cr3, eax
    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax
.no_paging:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov gs, ax

    mov ax, 0x28
    mov fs, ax

    .byte 0xEA
    .long .final_path
    .word 0x08

.final_path:
    popad
    pop ebp
    ret

.section .data
.align 4096
.global PAGING_BUFFER
PAGING_BUFFER: .fill 40960, 1, 0
// PAGING_BUFFER: .fill 81920, 1, 0
// PAGING_BUFFER: .fill 36864, 1, 0
