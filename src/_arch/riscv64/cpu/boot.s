.option norvc
.section .data
.section .text.init
.global	_start

_start:
	csrr	t0, mhartid
	bnez	t0, hart_parking_loop
.option norelax
	la 		gp, _global_pointer
	la		a0, _bss_start
	la 		a1, _bss_end
	bgeu	a0, a1, init_rust_state
clear_bss:
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, clear_bss
init_rust_state:
	la		sp, _stack
	li		t0, (0b11 << 11) | (1 << 7) | (1 << 3)
	csrw 	mstatus, t0
	la		t1, main
	csrw	mepc, t1
	la		t2, asm_trap_vector
	csrw 	mtvec, t2
	li		t3, (1 << 3) | (1 << 7) | (1 << 11)
	csrw 	mie, t3
	la 		ra, parking_loop
	mret
hart_parking_loop:
parking_loop:
	wfi
	j 		parking_loop
