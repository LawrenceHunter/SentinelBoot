//! Global Allocator
#![no_std]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

use alloc::string;
use console::interface::All;
use console::logln;
use core::alloc::*;
use core::ops::{DerefMut, Deref};
use core::ptr::null_mut;
use synchronisation::interface::Mutex;

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

/// Represents if the Alloc is free or allocated
#[repr(u8)]
#[derive(PartialEq)]
pub enum AllocFlags {
    /// Currently in use
    Allocated = 0,
    /// Available
    Free = 1,
}

/// Wrapper around the actual Alloc values
pub struct AllocInner {
    addr: usize,
    flags: AllocFlags,
}

pub struct AllocPointer {
    p: *mut Alloc
}
/// Every byte is described by the Alloc structure forming a linked list
pub struct Alloc {
    curr: synchronisation::NullLock<AllocInner>,
    prev: Option<synchronisation::NullLock<AllocPointer>>,
    next: Option<synchronisation::NullLock<AllocPointer>>,
}

unsafe impl Sync for Alloc {}

impl Deref for AllocPointer {
    type Target = Alloc;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.p) }
    }
}

impl DerefMut for AllocPointer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.p) }
    }
}

/// Embedded implementation for heap memory allocation
#[derive(Default)]
pub struct Allocator;

/// The static global allocator.
#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;

/// The static root Alloc.
static ROOT_ALLOC: Alloc = Alloc {
    curr: synchronisation::NullLock::new(AllocInner { addr: HEAP_START, flags: AllocFlags::Free }),
    prev: None,
    next: None,
};

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Adds helpful function implementations
impl AllocInner {
    /// Sets the internal address
    pub fn set_addr(&mut self, addr: usize) {
        self.addr = addr;
    }

    /// Gets the internal address
    pub fn addr(&self) -> usize {
        self.addr
    }

    /// Gets the internal flags
    pub fn flags(&self) -> AllocFlags {
        self.flags
    }

    /// Sets the internal flags
    pub fn set_flags(&mut self, flags: AllocFlags) {
        self.flags = flags;
    }
}

/// Implements functions to assist with memory allocation
impl Alloc {
    pub fn get_start_address(self) -> usize {
        self.curr.lock(|inner| inner.addr())
    }

    pub fn get_flags(self) -> AllocFlags {
        self.curr.lock(|inner| inner.flags())
    }

    pub fn set_address(self, addr: usize) {
        self.curr.lock(|inner| inner.set_addr(addr));
    }

    pub fn set_flags(self, flags: AllocFlags) {
        self.curr.lock(|inner| inner.set_flags(flags));
    }

    pub fn get_size(self) -> usize {
        let current_address = self.curr.lock(|inner| inner.addr());
        let mut address_width = 0;
        if let Some(x) = self.next {
            address_width = x.lock(|inner| *(inner)).deref().curr.lock(|inner| inner.addr());
        }
        address_width - current_address
    }

    pub fn get_end_address(self) -> usize {
        self.curr.lock(|inner| inner.addr()) + self.get_size()
    }

    pub fn set_value(self, addr: usize, value: u8) {
        assert!(addr >= self.get_start_address() && addr < self.get_end_address());
        unsafe { *(addr as *mut u8) = value; }
    }

    pub fn get_value(self, addr: usize) -> u8 {
        assert!(addr >= self.get_start_address() && addr < self.get_end_address());
        unsafe { return *(addr as *mut u8); }
    }

    pub fn get_prev(self) -> Option<synchronisation::NullLock<AllocPointer>> {
        self.prev
    }

    pub fn get_next(self) -> Option<synchronisation::NullLock<AllocPointer>> {
        self.next
    }
}

impl Allocator {
    /// Initialise the allocation system
    pub fn init() {
    }

    /// Returns the number of addresses marked taken
    pub fn get_alloc_count() -> usize {
        let mut temp_alloc = &ROOT_ALLOC;
        let mut count: usize = 0;
        while temp_alloc.get_next().is_some() {
            if temp_alloc.get_next().unwrap().lock(|inner| inner.deref().get_flags() == AllocFlags::Allocated) {
                count += temp_alloc.get_next().unwrap().lock(|inner| inner.deref().get_size());
            }
            temp_alloc = temp_alloc.get_next().unwrap().lock(|inner| inner.deref());
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
        let num_addresses = HEAP_SIZE / 8;
        let ptr = HEAP_START as *mut u8;

        // Find an alloc with enough bytes which is marked free
        let mut temp_alloc = &mut ROOT_ALLOC;
        while ((temp_alloc.get_size() < layout.size()) | (temp_alloc.get_flags() != AllocFlags::Free)) && temp_alloc.get_next().is_some() {
            temp_alloc = temp_alloc.get_next().unwrap().lock(|inner| inner.deref_mut());
        }

        // No memory was available
        if temp_alloc.get_flags() == AllocFlags::Allocated {
            return null_mut();
        }

        // Calculate the alloc boundary
        let new_end = temp_alloc.get_start_address() + layout.size();

        // Set the Alloc as allocated
        temp_alloc.set_flags(AllocFlags::Allocated);

        if let Some(mut x) = temp_alloc.get_next() {
            // If the next Alloc is free let's amalgamate the space
            if x.lock(|inner| inner.get_flags() == AllocFlags::Free) {
                x.lock(|inner| inner.set_address(new_end));
                return temp_alloc.get_start_address() as *mut u8;
            }
            // Else create a new free Alloc between
            else {
                let new_alloc_inner = synchronisation::NullLock::new(AllocInner {addr: new_end, flags: AllocFlags::Free});
                let prev_pointer = Some(synchronisation::NullLock::new(AllocPointer{p: temp_alloc as *mut Alloc}));
                let next_pointer = Some(x);
                let mut new_alloc = Alloc {curr: new_alloc_inner, prev: prev_pointer, next: next_pointer};
                let new_alloc_pointer = Some(synchronisation::NullLock::new(AllocPointer{p: (&mut new_alloc) as *mut Alloc}));
                temp_alloc.next = new_alloc_pointer;
                x.lock(|inner| inner.prev = new_alloc_pointer);
                return new_alloc.curr.lock(|inner| inner.addr()) as *mut u8;
            }
        }

        // If we don't have an Alloc after create a new one
        let new_alloc_inner = synchronisation::NullLock::new(AllocInner {addr: new_end, flags: AllocFlags::Free});
        let prev_pointer = Some(synchronisation::NullLock::new(AllocPointer{p: temp_alloc as *mut Alloc}));
        let mut new_alloc = Alloc {curr: new_alloc_inner, prev: prev_pointer, next: None};
        let new_alloc_pointer = Some(synchronisation::NullLock::new(AllocPointer{p: (&mut new_alloc) as *mut Alloc}));
        temp_alloc.next = new_alloc_pointer;
        return new_alloc.curr.lock(|inner| inner.addr()) as *mut u8;
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

        let mut p = ptr as *mut u8;
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
        let ptr = HEAP_START as *mut u8;

        // Each address is 16 bits so lets normalise to 16 bit chunks
        let normalised_size = layout.size();

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
        let ptr_clone = ptr as *mut u8;
        // Each address is 16 bits so lets normalise to 16 bit chunks
        let normalised_new_size = new_size;

        // Each address is 16 bits so lets normalise to 16 bit chunks
        let normalised_orig_size = layout.size();

        // Each address is 16 bits so lets normalise to 16 bit chunks
        let normalised_diff = normalised_new_size - normalised_orig_size;

        // No contiguous allocation was found
        null_mut()
    }
}

/// If there is an out of memory error, just panic.
#[alloc_error_handler]
fn allocator_error(_layout: Layout) -> ! {
    panic!("Memory allocation failed");
}
