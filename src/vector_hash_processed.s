li a0, 8
hash_kernel_asm:
.word 0x18572D7
.word 0x205F007
addi a1, a1, 64
addi a2, a2, -64
hash_kernel_asm_internal_loop:
.word 0x205F407
addi a1, a1, 64
addi a2, a2, -64
.word 0x205F807
addi a1, a1, 64
addi a2, a2, -64
.word 0xB7042077
bnez a2, hash_kernel_asm_internal_loop
