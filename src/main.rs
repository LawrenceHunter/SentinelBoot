//! The `bootloader` binary.
//!
//! 1. The bootloader's entry point is the function `cpu::boot::arch_boot::_start()`.
//!     - It is implemented in `src/_arch/__arch_name__/cpu/boot.s`.
//! 2. Once finished with architectural setup, the arch code calls `kernel_init()`.

#![allow(clippy::upper_case_acronyms)]
#![feature(naked_functions, asm_const, type_ascription)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![feature(format_args_nl)]
#![no_main]
#![no_std]

mod cpu;
mod panic_wait;
mod print;
use console::console;

/// Early init code.
///
/// # Safety
///
/// - Only a single hart must be active and running this function.
extern "C" fn loader_init() {
    // Initialise BSP driver subsystem
    if let Err(x) = unsafe { bsp::device_driver::init() } {
        panic!("Error intialising BSP driver subsystem: {}", x);
    }

    // Initialise all device drivers
    unsafe {
        driver::driver_manager().init_drivers();
    };
    // println! usable from here

    // Transition from unsafe to safe
    loader_main()
}

// Main function running after early init
fn loader_main() -> ! {
    unsafe {
        core::arch::asm!("lui a1, 0");
    }
    println!(
        "[0] {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    unsafe {
        core::arch::asm!("lui a1, 1");
    }

    println!("[1] Booting on: {}", bsp::board_name());
    unsafe {
        core::arch::asm!("lui a1, 2");
    }

    println!("[2] Drivers loaded:");
    unsafe {
        core::arch::asm!("lui a1, 3");
    }

    println!("[3] Chars written: {}", console().chars_written());
    unsafe {
        core::arch::asm!("lui a1, 4");
    }

    println!("[4] Echoing input now.");
    unsafe {
        core::arch::asm!("lui a1, 5");
    }

    console().clear_rx();
    unsafe {
        core::arch::asm!("lui a1, 6");
    }
    loop {
        let c = console().read_char();
        console().write_char(c);
    }
}
