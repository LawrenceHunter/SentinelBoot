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

mod assert_hex;
mod cpu;
mod helper;
mod panic_wait;
mod run_time_checks;

use core::{arch::asm, slice};

use bsp::bsp;
use console::{console, println};
use global_allocator::Allocator;
use sha2::{Digest, Sha256};

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

    println!("Chars written: {}", console().chars_written());

    if TEST {
        run_time_checks::suite();
    }

    unsafe {
        let mut data: u128;
        let mut address: usize = bsp::memory::map::kernel::KERNEL - 0x100;
        for _ in 0..10 {
            data = core::ptr::read(address as *mut u128);
            println!("{:#010x}: {:>#034x}", address, data);
            address += 0x10
        }
    }
    unsafe {
        let mut data: u128;
        let mut address: usize = bsp::memory::map::kernel::KERNEL;
        for _ in 0..10 {
            data = core::ptr::read(address as *mut u128);
            println!("{:#010x}: {:>#034x}", address, data);
            address += 0x10
        }
    }
    unsafe {
        let mut data: u128;
        let mut address: usize = bsp::memory::map::kernel::DTB;
        for _ in 0..10 {
            data = core::ptr::read(address as *mut u128);
            println!("{:#010x}: {:>#034x}", address, data);
            address += 0x10
        }
    }
    unsafe {
        let mut data: u128;
        let mut address: usize = bsp::memory::map::kernel::RAMFS;
        for _ in 0..10 {
            data = core::ptr::read(address as *mut u128);
            println!("{:#010x}: {:>#034x}", address, data);
            address += 0x10
        }
    }

    // Hash kernel
    let mut hasher = Sha256::new();
    println!("Instantiated hasher.");

    let mut offset = 0;
    let buff_size = 4096;
    loop {
        let data = unsafe {
            slice::from_raw_parts(
                (bsp::memory::map::kernel::KERNEL + (offset * buff_size))
                    as *mut u8,
                buff_size,
            )
        };
        // This is the problem how at runtime do I detect 13365248 bytes
        if (offset * buff_size) >= 13365248 {
            break;
        }
        hasher.update(data);
        offset += 1;
    }

    println!("Hasher update finished.");
    let hash = hasher.finalize();
    println!("Binary hash: \n{:X?}", hash);
    let expected: [u8; 32] = [
        0x6A, 0xD, 0x64, 0x99, 0x6D, 0x62, 0x74, 0xBA, 0x39, 0x8B, 0x5B, 0x64,
        0x18, 0x88, 0x5E, 0x9B, 0x2E, 0xF0, 0xA, 0x16, 0x45, 0xD7, 0xFB, 0x37,
        0x6C, 0x75, 0x76, 0xCF, 0xA, 0xC4, 0xA9, 0x53,
    ];
    println!("Expected hash: \n{:X?}", expected);
    assert_eq_hex!(expected, hash.as_slice());
    println!();

    let public_key = ed25519_compact::PublicKey::from_slice(crate::helper::PUBLIC_KEY).unwrap();
    println!("{:?}\n", public_key);
    let signature_bytes = unsafe { slice::from_raw_parts(
        (bsp::memory::map::kernel::KERNEL - 0x100)
            as *mut u8,
        64,
    ) };
    let signature = ed25519_compact::Signature::from_slice(signature_bytes).unwrap();
    println!("Signature {:?}\n", signature);
    let verified = public_key.verify(hash.as_slice(), &signature);
    println!("Verified: {:?}", verified);

    // println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
    // println!("X                          ENTERING MACHINE MODE
    // X");
    // println!("XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
    // unsafe {
    //     asm!("mret");
    // }
}
