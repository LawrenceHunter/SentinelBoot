#![no_std]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

use console::{log, logln};
use core::alloc::*;
use core::ptr::null_mut;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

// Used to mark the start of allocatable memory
static mut ALLOC_START: usize = 0;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Wraps address flag values
#[repr(u8)]
pub enum AddressBits {
    /// Not allocated
    Empty = 0,
    /// Allocated
    Taken = 1 << 0,
    /// Final page in table
    Last = 1 << 1,
}

/// Each byte is described by the Address structure
pub struct Address {
    flags: u8,
}

/// Embedded implementation for heap memory allocation
#[derive(Default)]
pub struct Allocator;

/// The static global allocator.
#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

impl AddressBits {
    /// Convert AddressBits to a u8
    pub fn val(self) -> u8 {
        self as u8
    }
}

impl Address {
    /// Returns if the address has been marked as the final allocation
    pub fn is_last(&self) -> bool {
        if self.flags & AddressBits::Last.val() != 0 {
            true
        } else {
            false
        }
    }

    /// Returns if the address is marked as being taken (allocated)
    pub fn is_taken(&self) -> bool {
        if self.flags & AddressBits::Taken.val() != 0 {
            true
        } else {
            false
        }
    }

    /// This is the opposite of is_taken().
    pub fn is_free(&self) -> bool {
        !self.is_taken()
    }

    /// Clear the address structure and all associated allocations
    pub fn clear(&mut self) {
        self.flags = AddressBits::Empty.val();
    }

    /// Set a certain flag
    pub fn set_flag(&mut self, flag: AddressBits) {
        self.flags |= flag.val();
    }

    /// Unset a certain flag
    pub fn clear_flag(&mut self, flag: AddressBits) {
        self.flags &= !(flag.val());
    }
}

impl Allocator {
    /// Initialise the allocation system
    pub fn init() {
        unsafe {
            let num_addresses = HEAP_SIZE / 8;
            let ptr = HEAP_START as *mut Address;
            // Clear all addresses
            for i in 0..num_addresses {
                (*ptr.add(i)).clear();
            }
        }
    }
    /// Print all page allocations
    /// This is mainly used for debugging.
    pub fn print_address_allocations() {
        unsafe {
            let num_addresses = HEAP_SIZE / 16;

            let mut start = HEAP_START as *const Address;
            let end = start.add(num_addresses);

            logln!("-----------------------------------------");
            logln!("BYTE ALLOCATION TABLE\nRANGE: {:p} -> {:p}", start, end);

            let mut num = 0;
            while start < end {
                if (*start).is_taken() {
                    let start_usize = start as usize;
                    let memaddr = ALLOC_START + (start_usize - HEAP_START);
                    log!("0x{:x} => ", memaddr);
                    while !(*start).is_last() {
                        num += 1;
                        start = start.add(1);
                    }
                    if (*start).is_last() {
                        let end = start as usize;
                        let memaddr = ALLOC_START + end;
                        logln!("FINAL: 0x{:x}", memaddr);
                    }
                }
                start = start.add(1);
            }
            logln!("ALLOC {:>9} BYTES", (num) * 2);
            logln!("FREE  {:>9} BYTES", (num_addresses - num) * 2);
            logln!("-----------------------------------------");
        }
    }
}

/// Allocate a byte or multiple bytes
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        assert!(layout.size() > 0);

        // We have to find a contiguous allocation of bytes
        // Create a Byte structure for each byte on the heap
        let num_addresses = HEAP_SIZE / 16;
        let ptr = HEAP_START as *mut Address;

        // Each address is 16 bits so lets normalise to 16 bit chunks
        let normalised_size = (layout.size() / 2) + 1;

        for i in 0..num_addresses - normalised_size {
            let mut found = false;
            // Check if the chunk is free
            if (*ptr.add(i)).is_free() {
                found = true;
                for j in i..i + normalised_size {
                    // Check to see if we have contiguous allocation
                    if (*ptr.add(j)).is_taken() {
                        found = false;
                        break;
                    }
                }
            }
            if found {
                for k in i..i + normalised_size - 1 {
                    (*ptr.add(k)).set_flag(AddressBits::Taken);
                }
                (*ptr.add(i + normalised_size - 1))
                    .set_flag(AddressBits::Taken);
                (*ptr.add(i + normalised_size - 1)).set_flag(AddressBits::Last);
                logln!(
                    "(Allocated) {:>11} bytes: {:<10?} -> {:<10?}",
                    layout.size(),
                    ptr.add(i),
                    ptr.add(i + normalised_size - 1)
                );
                return ptr.add(i) as *mut u8;
            }
        }
        // No contiguous allocation was found
        null_mut()
    }

    /// Deallocate a byte by its pointer
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Ensure we don't free a null pointer.
        assert!(!ptr.is_null());

        // Make sure that the address makes sense
        assert!(
            (ptr as usize) >= HEAP_START
                && (ptr as usize) < HEAP_START + HEAP_SIZE
        );

        let mut p = ptr as *mut Address;

        // Keep clearing bytes until we hit the last byte.
        while (*p).is_taken() && !(*p).is_last() {
            (*p).clear();
            p = p.add(1);
        }
        assert!((*p).is_last() == true, "Possible double-free detected!");
        (*p).clear();
        logln!(
            "(Deallocated) {:>9} bytes: {:<10?} -> {:<10?}",
            layout.size(),
            ptr,
            ptr.add(layout.size() / 2)
        );
    }
}

/// If there is an out of memory error, just panic.
#[alloc_error_handler]
fn allocator_error(_layout: Layout) -> ! {
    Allocator::print_address_allocations();
    panic!("Memory allocation failed");
}
