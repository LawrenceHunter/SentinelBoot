# trap.S
# In the future our trap vector will go here.

.section .text
.global asm_trap_vector
# This will be our trap vector when we start
# handling interrupts.
.align 4
asm_trap_vector:
	wfi
	j asm_trap_vector
