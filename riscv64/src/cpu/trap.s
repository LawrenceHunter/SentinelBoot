# trap.S
# Assembly-level trap handler.

.global asm_trap_vector
asm_trap_vector:
    # We get here when the CPU is interrupted
	# for any reason.
    mret
