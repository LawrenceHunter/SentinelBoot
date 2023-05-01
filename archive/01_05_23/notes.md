# 010523

- Initial branch to 0x8000_0000 removed
- Clearly gets stuck at the end which is a nop instruction likely mine
- 0x80000500 area is heavily branched

```gdb
(gdb) x/40i 0x80000500
   0x80000500:	addi	a2,sp,732
   0x80000502:	li	a7,240
   0x80000506:	lui	a6,0x110
   0x8000050a:	j	0x8000051c
   0x8000050c:	addi	a1,a1,1
   0x8000050e:	ld	a2,0(a0)
   0x80000510:	sw	a5,4(a2)
   0x80000512:	ld	a2,8(a0)
   0x80000514:	addi	a2,a2,1
   0x80000516:	sd	a2,8(a0)
   0x80000518:	beq	a1,t1,0x80000578
   0x8000051c:	lb	a3,0(a1)
   0x80000520:	zext.b	a5,a3
   0x80000524:	bgez	a3,0x8000050c
   0x80000528:	lbu	a3,1(a1)
   0x8000052c:	andi	a4,a5,31
   0x80000530:	andi	a3,a3,63
   0x80000534:	bgeu	t0,a5,0x80000564
   0x80000538:	lbu	a2,2(a1)
   0x8000053c:	slli	a3,a3,0x6
   0x8000053e:	andi	a2,a2,63
   0x80000542:	or	a3,a3,a2
   0x80000544:	bltu	a5,a7,0x8000056e
   0x80000548:	lbu	a2,3(a1)
   0x8000054c:	slli	a4,a4,0x3d
   0x8000054e:	slli	a3,a3,0x6
   0x80000550:	andi	a2,a2,63
   0x80000554:	srli	a4,a4,0x2b
   0x80000556:	or	a2,a2,a3
   0x80000558:	or	a5,a2,a4
   0x8000055c:	beq	a5,a6,0x80000578
   0x80000560:	addi	a1,a1,4
   0x80000562:	j	0x8000050e
   0x80000564:	slli	a4,a4,0x6
   0x80000566:	addi	a1,a1,2
   0x80000568:	or	a5,a4,a3
   0x8000056c:	j	0x8000050e
   0x8000056e:	slli	a4,a4,0xc
   0x80000570:	addi	a1,a1,3
   0x80000572:	or	a5,a3,a4
```

Initial impressions are the bootloader is working correctly but there is a bug within UART the branching and repeated patterns seem to indicate it is attempting to print and the tight branching in the centre of the graph indicates a waiting state.

- https://osblog.stephenmarz.com/ch2.html has proven a useful resource
- http://caro.su/msx/ocm_de1/16550.pdf

## UART Dive
Using gdb to break on write to the uart memory location the program never writes to 0x10000000-0x10000008. My earlier point is disproved the program fails before UART init().

Adding an instruction to main() to see if we get that far. This does occur.

Adding a fake instruction to mmio deref to see if we get that far. This does occur.

Adding a fake instruction to device driver init to see if we get that far. This does occur.

Adding a fake instruction to the start of our code to see if we get that far. This does not occur. There is a problem between driver instantiation and the start of main.

Start of loader init is ran.

Error occurs in `driver::driver_manager().init_drivers();`

Error occurs in:

```rust
if let Err(x) = descriptor.device_driver.init() {
      panic!(
         "Error initialising driver: {}: {}",
         descriptor.device_driver.compatible(), x
      );
}
```

Error found: `self.registers.LCR.write(LCR::DLAB::Disabled);`
Seems the addressing of the register locations was incorrect and needs to be offset by word i.e. +4 not +1.
We can now finish `driver::driver_manager().init_drivers();`
We now reach `loader_main()`

```rust
    println!(
        "[0] {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
```
Seems to fail.

It appears we don't reach "fn write_char(&mut self, c: char)"
Error occurs in `console::console().write_fmt(args).unwrap();`
My current understanding of the error is that the overwriting of the console implementation by the driver implementing the trait does not occur properly it seems we reach the print function but the call to format_args fails very quickly. Narrowing down where we end up it is `asm_trap_vector`.
