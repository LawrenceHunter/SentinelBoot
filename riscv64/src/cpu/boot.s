// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>
// Copyright (c) 2023-2024 Lawrence Hunter <lawrence.hunter@outlook.com>

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
	csrw	satp, zero
	# Any harts not bootstrapping need to wait for IPI
	csrr	t0, mhartid
	bnez 	t0, hart_parking_loop

	# Set all bytes in BSS to 0
	la		a0, _bss_start
	la 		a1, _bss_end
	bgeu	a0, a1, init_rust_state
clear_bss:
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, clear_bss
init_rust_state:
	la		sp, _stack

	# Set mstatus to MPP
	li		t0, 0b11 << 11
	csrw 	mstatus, t0

	# Do not allow interrupts while running main
	csrw 	mie, zero

	# MEPC set to main
	la		t1, main
	csrw	mepc, t1

	# Set return address to enter supervisor mode
	la 		ra, parking_loop
	mret
hart_parking_loop:
	# Wait here until we recieve a software interrupt

	# Divide the stack among harts
	la 		sp, _stack
	li		t0, 0x1000
	csrr	a0, mhartid
	mul		t0, t0, a0
	sub		sp, sp, t0

	# Put harts into machine mode with interrupts enabled
	li 		t0, 0b11 << 11 | (1 << 7)
	csrw 	mstatus, t0

	la		t1, main_hart
	csrw	mepc, t1

	la 		t2, asm_trap_vector
	csrw 	mtvec, t2

	la 		ra, parking_loop
	mret
parking_loop:
	wfi
	j 		parking_loop
