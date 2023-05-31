//! The `bootloader` binary.
//!
//! 1. The bootloader's entry point is the function
//! `cpu::boot::arch_boot::_start()`.
//!     - It is implemented in `src/_arch/__arch_name__/cpu/boot.s`.
//! 2. Once finished with architectural setup, the arch code calls
//! `kernel_init()`.

#![allow(clippy::upper_case_acronyms)]
#![feature(naked_functions, asm_const, type_ascription)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![feature(format_args_nl)]
#![no_main]
#![no_std]

mod cpu;
mod helper;
mod panic_wait;
use console::{console, println};

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

#[no_mangle]
extern "C" fn main_hart(_hartid: usize) {
    // We aren't going to do anything here until we get SMP going.
    // All non-0 harts initialize here.
}

// Main function running after early init
fn loader_main() -> ! {
    crate::helper::print_boot_log();

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
