//! Global Allocator
#![no_std]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

use console::{logln, println};
use core::alloc::*;
use core::ptr::null_mut;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Wraps address flag values
#[repr(u16)]
pub enum AddressBits {
    /// Not allocated
    Empty = 0b00 << 8,
    /// Allocated
    Taken = 0b01 << 8,
    /// Final page in table
    Last = 0b10 << 8,
}

/// Every byte is described by the Address structure
/// This wastes 50% of storage but we can't be more precise than the address which is u16 but
pub struct Address {
    flags: u16,
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
    pub fn val(self) -> u16 {
        self as u16
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
        self.flags &= !(flag.val()) & 0xff;
    }
}

impl Allocator {
    /// Initialise the allocation system
    pub fn init() {
        unsafe {
            let num_addresses = HEAP_SIZE / 16;
            let ptr = HEAP_START as *mut Address;
            logln!(
                "CLEARING ADDRESSES {:?} -> {:?}.",
                ptr,
                ptr.add(num_addresses)
            );
            // Clear all addresses
            for i in 0..num_addresses {
                (*ptr.add(i)).clear();
            }
            logln!("LOGGING FIRST 10 CLEARS:");
            for i in 0..10 {
                logln!("\tMARKED ADDRESS {:?} AS FREE", ptr.add(i));
            }
        }
    }

    /// Returns the number of addresses marked taken
    pub fn get_alloc_count() -> u64 {
        let mut count = 0;
        unsafe {
            let num_addresses = HEAP_SIZE / 16;
            let mut start = HEAP_START as *const Address;
            let end = start.add(num_addresses);
            while start < end {
                if (*start).is_taken() {
                    count += 1;
                }
                start = start.add(1);
            }
        }
        count
    }
}

/// Allocate a byte or multiple bytes
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        assert!(layout.size() > 0);
        logln!("ALLOCATING {:>8} BYTES", layout.size());

        // We have to find a contiguous allocation of bytes
        // Create a Byte structure for each byte on the heap
        let num_addresses = HEAP_SIZE / 16;
        let ptr = HEAP_START as *mut Address;

        // Each address is 16 bits so lets normalise to 16 bit chunks
        let normalised_size = layout.size();

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
                    logln!("\tMARKING {:<10?} TAKEN", ptr.add(k));
                }
                (*ptr.add(i + normalised_size - 1))
                    .set_flag(AddressBits::Taken);
                logln!(
                    "\tMARKING {:<10?} TAKEN",
                    ptr.add(i + normalised_size - 1)
                );
                (*ptr.add(i + normalised_size - 1)).set_flag(AddressBits::Last);
                logln!(
                    "\tMARKING {:<10?} LAST",
                    ptr.add(i + normalised_size - 1)
                );
                logln!(
                    "ALLOCATED {:>9} BYTES: {:<10?} -> {:<10?}",
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

        logln!(
            "DEALLOCATING {:>7} BYTES: {:<10?} -> {:<10?}",
            layout.size(),
            ptr,
            ptr.add(layout.size())
        );

        // Make sure that the address makes sense
        assert!(
            (ptr as usize) >= HEAP_START
                && (ptr as usize) < HEAP_START + HEAP_SIZE
        );

        let mut p = ptr as *mut Address;

        // Keep clearing bytes until we hit the last byte.
        while (*p).is_taken() && !(*p).is_last() {
            logln!("\tMARKING {:<10?} FREE", p);
            (*p).clear();
            *(p as *mut u8) = 0;
            p = p.add(1);
        }
        assert!((*p).is_last() == true, "Possible double-free detected!");
        (*p).clear();
        logln!(
            "DEALLOCATED {:>7} BYTES: {:<10?} -> {:<10?}",
            layout.size(),
            ptr,
            ptr.add(layout.size())
        );
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        assert!(layout.size() > 0);
        logln!("ZERO ALLOCATING {:>9} BYTES", layout.size());

        // We have to find a contiguous allocation of bytes
        // Create a Byte structure for each byte on the heap
        let num_addresses = HEAP_SIZE / 16;
        let ptr = HEAP_START as *mut Address;

        // Each address is 16 bits so lets normalise to 16 bit chunks
        let normalised_size = layout.size();

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
                    *(ptr.add(k) as *mut u8) = 0;
                    logln!("\tMARKING {:<10?} TAKEN", ptr.add(k));
                }
                logln!(
                    "\tMARKING {:<10?} TAKEN",
                    ptr.add(i + normalised_size - 1)
                );
                (*ptr.add(i + normalised_size - 1)).set_flag(AddressBits::Last);
                logln!(
                    "\tMARKING {:<10?} LAST",
                    ptr.add(i + normalised_size - 1)
                );
                *(ptr.add(i + normalised_size - 1) as *mut u8) = 0;
                logln!(
                    "ZERO ALLOCATED {:>4} BYTES: {:<10?} -> {:<10?}",
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

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize,
    ) -> *mut u8 {
        assert!(new_size > 0);
        logln!(
            "REALLOCATING {:?} FROM {} BYTES TO {} BYTES",
            ptr,
            layout.size(),
            new_size
        );
        let ptr_clone = ptr as *mut Address;
        // Each address is 16 bits so lets normalise to 16 bit chunks
        let normalised_new_size = new_size;

        // Each address is 16 bits so lets normalise to 16 bit chunks
        let normalised_orig_size = layout.size();

        // Each address is 16 bits so lets normalise to 16 bit chunks
        let normalised_diff = normalised_new_size - normalised_orig_size;

        let mut end_ptr = ptr as *mut Address;
        end_ptr = end_ptr.add(normalised_orig_size);

        let mut extend = true;
        for i in 0..normalised_diff {
            // Check if the chunk is free
            logln!(
                "\tCHECKING IF ADDRESS {:?} IS MARKED FREE.",
                end_ptr.add(i)
            );
            if !(*end_ptr.add(i)).is_free() {
                logln!("\t\tADDRESS {:?} IS MARKED TAKEN.", end_ptr.add(i));
                extend = false;
            }
        }

        // If we can just extend the current allocation
        if extend {
            println!("EXTENDING:");
            let mut i = normalised_orig_size;
            while i < normalised_new_size - 1 {
                (*ptr_clone.add(i)).clear();
                (*ptr_clone.add(i)).set_flag(AddressBits::Taken);
                *(ptr.add(i) as *mut u8) = 0;
                logln!("\tMARKING {:<10?} TAKEN", ptr.add(i));
                i += 1
            }
            (*ptr_clone.add(i + normalised_new_size - 1))
                .set_flag(AddressBits::Taken);
            (*ptr_clone.add(i + normalised_new_size - 1))
                .set_flag(AddressBits::Last);
            logln!(
                "\tMARKING {:<10?} TAKEN",
                ptr.add(i + normalised_new_size - 1)
            );
            logln!(
                "\tMARKING {:<10?} LAST",
                ptr.add(i + normalised_new_size - 1)
            );
            logln!(
                "REALLOCATED {:>7} BYTES: {:<10?} -> {:<10?}",
                new_size,
                ptr_clone,
                ptr_clone.add(i + normalised_new_size - 1)
            );
            return ptr_clone as *mut u8;
        }
        // If we have to allocate a new chunk
        else {
            println!("RELOCATING:");
            let old_ptr = ptr as *mut Address;

            // We have to find a contiguous allocation of bytes
            // Create a Byte structure for each byte on the heap
            let num_addresses = HEAP_SIZE / 16;
            let ptr = HEAP_START as *mut Address;

            for i in 0..num_addresses - normalised_new_size {
                let mut found = false;
                // Check if the chunk is free
                if (*ptr.add(i)).is_free() {
                    found = true;
                    for j in i..i + normalised_new_size {
                        // Check to see if we have contiguous allocation
                        if (*ptr.add(j)).is_taken() {
                            found = false;
                            break;
                        }
                    }
                }
                if found {
                    // Mark as taken
                    for k in i..i + normalised_new_size - 1 {
                        (*ptr.add(k)).set_flag(AddressBits::Taken);
                        *(ptr.add(k) as *mut u8) = 0;
                        logln!("\tMARKING {:<10?} TAKEN", ptr.add(k));
                    }

                    // Copy values from old alloc and free
                    for k in i..i + normalised_orig_size {
                        (*old_ptr.add(k)).clear();
                        logln!("\tMARKING {:<10?} FREE", old_ptr.add(k - i));
                        *(ptr.add(k) as *mut u8) =
                            *(old_ptr.add(k - i) as *mut u8);
                        logln!(
                            "\tMoving value 0x{:x} from {:?} to {:?}.",
                            *(old_ptr.add(k - i) as *mut u8),
                            old_ptr.add(k - i),
                            ptr.add(k)
                        )
                    }

                    // Set final address as last
                    (*ptr.add(i + normalised_new_size - 1))
                        .set_flag(AddressBits::Taken);
                    (*ptr.add(i + normalised_new_size - 1))
                        .set_flag(AddressBits::Last);
                    logln!(
                        "\tMARKING {:<10?} TAKEN",
                        ptr.add(i + normalised_new_size - 1)
                    );
                    logln!(
                        "\tMARKING {:<10?} LAST",
                        ptr.add(i + normalised_new_size - 1)
                    );

                    logln!(
                        "REALLOCATED {:>7} BYTES: {:<10?} -> {:<10?}",
                        layout.size(),
                        ptr.add(i),
                        ptr.add(i + normalised_new_size - 1)
                    );
                    return ptr.add(i) as *mut u8;
                }
            }
        }
        // No contiguous allocation was found
        null_mut()
    }
}

/// If there is an out of memory error, just panic.
#[alloc_error_handler]
fn allocator_error(_layout: Layout) -> ! {
    panic!("Memory allocation failed");
}
