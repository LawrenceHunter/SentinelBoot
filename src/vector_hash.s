# SEW = e64 (64 bytes)
# LMUL = 1
# a0 contains length of vector
# a1 contains pointer to kernel
# a2 contains size of kernel
# I think as SHA-256 I'll need a0 to be 4 not 8
li a0, 4
add a1, s1, zero
# add a2, {kernel_size}, zero
li a2, 96
# Sanity check forms a loop counter
li a3, 0
mv a4, v0
mv a5, v1
mv a6, v2
mv a7, v3
hash_kernel_asm:
    # Set vector length to 4 64 bit elements
    vsetvli t0, a0, e64, m1, tu, mu
    # Load 4 consecutive 64 bit values into v0-v7
    vle64.v v0, (a1)
    # Increment a1 to point to the next 64 byte chunk
    addi a1, a1, 64
    # Decrement a2 to reduce remaining size by 64 bytes
    addi a2, a2, -64
    hash_kernel_asm_internal_loop:
        add a3, a3, 1
        # Load 4 consecutive 64 bit values into v8-v15
        vle64.v v4, (a1)
        # Increment a1 to point to the next 32 byte chunk
        addi a1, a1, 32
        # Decrement a2 to reduce remaining size by 32 bytes
        addi a2, a2, -32
        # Load 4 consecutive 64 bit values into v16-v23
        vle64.v v8, (a1)
        # Increment a1 to point to the next 32 byte chunk
        addi a1, a1, 32
        # Decrement a2 to reduce remaining size by 32 bytes
        addi a2, a2, -32
        # vsha2ms.vv v0, v8, v4
        # 101101 1 10000 01000 010 00000 1110111
        .word 0xB7042077
    blt a2, zero, hash_kernel_asm_internal_loop
add s2, a3, zero
mv v0, a4
mv v1, a5
mv v2, a6
mv v3, a7
