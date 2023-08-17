.option norvc
.section .text.init
.global	_start

# Execution starts here
_start:
	# Disable linker relaxation for la
	# This disallows the assembler from assuming 'gp' is initialised
.option push
.option norelax
	la 		gp, _global_pointer
.option pop
	# Mask all interrupts
	csrw 	sie, zero

	# Set all bytes in BSS to 0
	la		a0, _bss_start
	la 		a1, _bss_end
	bgeu	a0, a1, init_rust_state
clear_bss:
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, clear_bss
init_rust_state:
	# Divide the stack among harts
	la 		sp, _stack
	li		t0, 0x1000
	li		a0, 1

	mul		t0, t0, a0
	addi 	a0, a0, 1
	sub		sp, sp, t0
	mul		t0, t0, a0
	addi 	a0, a0, 1
	sub		sp, sp, t0
	mul		t0, t0, a0
	addi 	a0, a0, 1
	sub		sp, sp, t0
	mul		t0, t0, a0
	addi 	a0, a0, 1
	sub		sp, sp, t0

	# Set trap vector
	la 		t2, asm_trap_vector
	csrw 	stvec, t2

	# Set sstatus to MPP
	li		t0, 0b01 << 11
	csrw 	sstatus, t0

	# MEPC set to main
	la		t1, main
	csrw	sepc, t1

	# Set return address to enter supervisor mode
	la 		ra, parking_loop
	#! Following with break points we reach rust
	sret
parking_loop:
	# Wait here until we receive a software interrupt
	wfi
	j 		parking_loop
