li a0, 4
add a1, {KERNEL_POINTER}, zero
li a2, 96
li a3, 0
hash_kernel_asm:
.word 0x18572D7
.word 0x205F007
addi a1, a1, 64
addi a2, a2, -64
hash_kernel_asm_internal_loop:
add a3, a3, 1
.word 0x205F207
addi a1, a1, 32
addi a2, a2, -32
.word 0x205F407
addi a1, a1, 32
addi a2, a2, -32
.word 0xB6822077
blt a2, zero, hash_kernel_asm_internal_loop
add {counter}, a3, zero
.word 0x70033
.word 0x178033
.word 0x280033
.word 0x388033
.word 0xE00033
.word 0xF08033
.word 0x1010033
.word 0x1118033
