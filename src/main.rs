//! The `bootloader` binary.
//!
//! 1. The bootloader's entry point is the function `cpu::boot::arch_boot::_start()`.
//!     - It is implemented in `src/_arch/__arch_name__/cpu/boot.s`.
//! 2. Once finished with architectural setup, the arch code calls `kernel_init()`.

#![allow(clippy::upper_case_acronyms)]
#![feature(naked_functions, asm_const, type_ascription)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

mod bsp;
mod console;
mod cpu;
mod driver;
mod panic_wait;
mod print;
mod synchronisation;

/// Early init code.
///
/// # Safety
///
/// - Only a single hart must be active and running this function.
unsafe fn loader_init() -> ! {
    // Initialise BSP driver subsystem
    if let Err(x) = bsp::driver::init() {
        panic!("Error intialising BSP driver subsystem: {}", x);
    }

    // Initialise all device drivers
    driver::driver_manager().init_drivers();
    // println! usable from here

    // Transition from unsafe to safe
    loader_main()
}

// Main function running after early init
fn loader_main() -> ! {
    use console::console;
    println!(
        "[0] {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    println!("[1] Booting on: {}", bsp::board_name());

    println!("[2] Drivers loaded:");
    driver::driver_manager().enumerate();

    println!("[3] Chars written: {}", console().chars_written());

    println!("[4] Echoing input now.");

    console().clear_rx();
    loop {
        let c = console().read_char();
        console().write_char(c);
    }
}
