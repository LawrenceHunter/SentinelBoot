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

extern crate alloc;

mod cpu;
mod helper;
mod panic_wait;
use alloc::vec::Vec;
use console::{logln, println};
use global_allocator::Allocator;

/// Early init code.
///
/// # Safety
///
/// - Only a single hart must be active and running this function.
extern "C" fn loader_init() {
    // Initialise BSP driver subsystem
    if let Err(x) = unsafe { bsp::device_driver::init() } {
        panic!("Error initialising BSP driver subsystem: {}", x);
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
    // All non-0 harts initialise here.
}

// Main function running after early init
fn loader_main() {
    // ########################################################################
    // ENSURE THESE LINES ARE FIRST
    crate::helper::print_boot_logo();
    Allocator::init();
    // ########################################################################

    println!(
        "{} version {} ({})",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        crate::helper::SHA
    );

    bsp::print_info();

    println!("Drivers loaded:");
    driver::driver_manager().enumerate();

    println!("Testing memory allocation:");
    {
        let mut x: Vec<u8> = Vec::new();
        for i in 0..10 as u8 {
            x.push(i);
            logln!("ADDRESSES ALLOCATED: {}", Allocator::get_alloc_count());
            logln!("Vector: {:?}", x);
        }
        logln!("ADDRESSES ALLOCATED: {}", Allocator::get_alloc_count());
        logln!("Vector: {:?}", x);
    }
    logln!("ADDRESSES ALLOCATED: {}", Allocator::get_alloc_count());
}
