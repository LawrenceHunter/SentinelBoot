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

#![allow(named_asm_labels)]
// #![allow(dead_code)]

extern crate alloc;

mod assert_hex;
mod cpu;
mod helper;
mod panic_wait;
mod run_time_checks;
mod verification;

#[cfg(feature = "qemu")]
mod vector_hash;

use core::arch::asm;

use bsp::bsp;
use console::println;
use global_allocator::Allocator;

static TEST: bool = false;

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

fn loader_machine() {
    println!("Handing execution to the kernel...");
    unsafe {
        // https://github.com/torvalds/linux/blob/master/Documentation/riscv/boot.rst
        // SATP expected to be 0
        // HARTID of current core needs to be in a0
        // FDT address needs to be in a1
        asm!(
            "li t0, 0",
            "csrw satp, t0",
            "li a0, {x}",
            "li a1, {y}",
            "li a2, {z}",
            "jalr x0, 0x0(a2)",
            x = const bsp::memory::map::kernel::HART,
            y = const bsp::memory::map::kernel::DTB,
            z = const bsp::memory::map::kernel::KERNEL
        );
    }
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

    if TEST {
        run_time_checks::suite();
    }

    match verification::verify_kernel() {
        Ok(_) => {
            println!("Loaded kernel hash matches signed hash proceeding...");
        }
        Err(_) => {
            panic!("! -- LOADED KERNEL HASH DOES NOT MATCH SIGNED HASH")
        }
    }

    unsafe {
        asm!("mret");
    }
}
