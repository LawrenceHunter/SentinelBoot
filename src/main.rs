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
mod run_time_checks;

use core::arch::asm;

use console::{console, println};
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
    println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
    println!("X                             IN MACHINE MODE                             X");
    println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
    unsafe {
        // https://github.com/torvalds/linux/blob/master/Documentation/riscv/boot.rst
        // SATP expected to be 0
        // HARTID of current core needs to be in a0
        // FDT address needs to be in a1
        asm!(
            "li t0, 0",
            "csrw satp, t0",
            "li a0, 0",
            "li a1, 0x84a00000",
            "li a2, 0x80200000",
            "jalr x0, 0x0(a2)"
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

    println!("Chars written: {}", console().chars_written());

    if TEST {
        run_time_checks::suite();
    }

    // unsafe {
    //     let mut data: u128;
    //     let mut address: usize = 0x80200000;
    //     for _ in 0..10 {
    //         data = core::ptr::read(address as *mut u128);
    //         println!("{:#010x}: {:>#034x}", address, data);
    //         address = address + 0x10;
    //     }
    // }
    // unsafe {
    //     let mut data: u128;
    //     let mut address: usize = 0x82A00000;
    //     for _ in 0..10 {
    //         data = core::ptr::read(address as *mut u128);
    //         println!("{:#010x}: {:>#034x}", address, data);
    //         address = address + 0x10;
    //     }
    // }
    // unsafe {
    //     let mut data: u128;
    //     let mut address: usize = 0x83000000;
    //     for _ in 0..10 {
    //         data = core::ptr::read(address as *mut u128);
    //         println!("{:#010x}: {:>#034x}", address, data);
    //         address = address + 0x10;
    //     }
    // }

    println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
    println!("X                          ENTERING MACHINE MODE                          X");
    println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
    unsafe {
        asm!("mret");
    }
}
