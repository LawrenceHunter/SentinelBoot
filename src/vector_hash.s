# SEW = e64 (64 bytes)
# LMUL = 1
# a0 contains length of vector
# a1 contains pointer to kernel
# a2 contains size of kernel
# I think as SHA-256 I'll need a0 to be 4 not 8
li a0, 8
hash_kernel_asm:
    # Set vector length to 8 64 bit elements
    vsetvli t0, a0, e64, m1, tu, mu
    # Load 8 consecutive 64 bit values into v0-v7
    vle64.v v0, (a1)
    # Increment a1 to point to the next 64 byte chunk
    add a1, a1, 64
    # Decrement a2 to reduce remaining size by 64 bytes
    sub a2, a2, 64
    hash_kernel_asm_internal_loop:
        # Load 8 consecutive 64 bit values into v8-v15
        vle64.v v8, (a1)
        # Increment a1 to point to the next 64 byte chunk
        add a1, a1, 64
        # Decrement a2 to reduce remaining size by 64 bytes
        sub a2, a2, 64
        # Load 8 consecutive 64 bit values into v16-v23
        vle64.v v16, (a1)
        # Increment a1 to point to the next 64 byte chunk
        add a1, a1, 64
        # Decrement a2 to reduce remaining size by 64 bytes
        sub a2, a2, 64
        # vsha2ms.vv v0, v16, v8
        # 101101 1 10000 01000 010 00000 1110111
        .word 0xB7042077
    bnez a2, hash_kernel_asm_internal_loop
