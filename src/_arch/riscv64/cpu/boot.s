//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
.section .text._start

//------------------------------------------------------------------------------
// fn _start()
//------------------------------------------------------------------------------
_start:
	li		x0, %lo(__bss_start)
	addi	x0, x0, %lo(__bss_start)
	li		x1, %lo(__bss_end_exclusive)
	addi	x1, x1, %lo(__bss_end_exclusive)

.L_bss_init_loop:
	beq 	x0, x1, .L_prepare_rust
	xor		x0, x0, x0
	j 		.L_bss_init_loop

.L_prepare_rust:
	li		x0, %lo(__boot_core_stack_end_exclusive)
	add		x0, x0, %lo(__boot_core_stack_end_exclusive)
	ori		sp, x0, 0
	j 		_start_rust

.size	_start, . - _start
.type	_start, function
.global	_start
