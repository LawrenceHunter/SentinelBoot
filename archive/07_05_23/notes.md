# 070523

Continuing from last week:

> It appears we don't reach "fn write_char(&mut self, c: char)"
Error occurs in `console::console().write_fmt(args).unwrap();`
My current understanding of the error is that the overwriting of the console implementation by the driver implementing the trait does not occur properly it seems we reach the print function but the call to format_args fails very quickly. Narrowing down where we end up it is `asm_trap_vector`.

We reach the uart driver's inner lock.
We hit the trap before `fn write_char(&mut self, c: char)`
My added assembly debugging caused this. We now reach `write_char` which now completes the register setting. println no longer errors.
As the program managed to reach the end of its main function it seemed suspicious that just my code was to blame as something was expected after fidling with QEMUs args output was achieved within docker but not natively on mac an issue will be opened to investigate this.
