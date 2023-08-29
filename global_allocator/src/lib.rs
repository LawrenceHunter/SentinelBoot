//! Global Allocator
#![no_std]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]

use console::logln;
use core::alloc::*;
use core::ops::{DerefMut, Deref};
use core::ptr::null_mut;
use synchronisation::interface::Mutex;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

// extern "C" {
//     /// Start of HEAP address space
//     static HEAP_START: usize;
//     /// Size of HEAP address space
//     static HEAP_SIZE: usize;
// }

// Temporary until I can get help with linking ^
static HEAP_START: usize = ;
static HEAP_SIZE: usize = ;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Represents if the Alloc is free or allocated
#[repr(u8)]
#[derive(PartialEq, Clone, Copy)]
pub enum AllocFlags {
    /// Currently in use
    Allocated = 0,
    /// Available
    Free = 1,
}

/// Wrapper around the actual Alloc values
#[derive(Clone, Copy)]
pub struct AllocInner {
    addr: usize,
    flags: AllocFlags,
}

/// Wrapper around the Alloc pointer
#[derive(Clone, Copy)]
pub struct AllocPointer {
    p: *mut Alloc
}

/// Every byte is described by the Alloc structure forming a linked list
pub struct Alloc {
    curr: synchronisation::NullLock<AllocInner>,
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
static mut GLOBAL_ALLOCATOR: Allocator = Allocator;

/// The static root Alloc.
static mut ROOT_ALLOC: Alloc = Alloc {
    curr: unsafe { synchronisation::NullLock::new(AllocInner { addr: HEAP_START, flags: AllocFlags::Free }) },
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
    /// Gets the curr address
    pub fn get_start_address(&self) -> usize {
        self.curr.lock(|inner| inner.addr())
    }

    /// Gets the curr flags
    pub fn get_flags(&self) -> AllocFlags {
        self.curr.lock(|inner| inner.flags())
    }

    /// Sets the curr address
    pub fn set_address(&mut self, addr: usize) {
        self.curr.lock(|inner| inner.set_addr(addr));
    }

    /// Sets the Alloc flags
    pub fn set_flags(&mut self, flags: AllocFlags) {
        self.curr.lock(|inner| inner.set_flags(flags));
    }

    /// Gets the size of the Alloc
    pub fn get_size(&self) -> usize {
        let current_address = self.curr.lock(|inner| inner.addr());
        let mut address_width = 0;
        if let Some(x) = &self.next {
            address_width = x.lock(|inner| *(inner)).deref().curr.lock(|inner| inner.addr());
        }
        address_width - current_address
    }

    /// Gets the end of the Alloc range
    pub fn get_end_address(&self) -> usize {
        self.curr.lock(|inner| inner.addr()) + self.get_size()
    }

    /// Sets the value at the stored address
    pub fn set_value(&mut self, addr: usize, value: u8) {
        assert!(addr >= self.get_start_address() && addr < self.get_end_address());
        unsafe { *(addr as *mut u8) = value; }
    }

    /// Gets the value at the stored address
    pub fn get_value(&self, addr: usize) -> u8 {
        assert!(addr >= self.get_start_address() && addr < self.get_end_address());
        unsafe { return *(addr as *mut u8); }
    }

    /// Checks if next is Some
    pub fn has_next(&self) -> bool {
        self.next.is_some()
    }

    /// Returns the unwrapped pointer or panic. This unwrapping passes the responsibility to the programmer.
    pub fn get_next(&self) -> &Alloc {
        assert!(self.has_next());
        return self.next.as_ref().unwrap().lock(|inner| inner);
    }

    /// Returns the unwrapped pointer or panic. This unwrapping passes the responsibility to the programmer.
    pub fn get_next_mut(&mut self) -> &mut Alloc {
        assert!(self.has_next());
        return self.next.as_ref().unwrap().lock(|inner| inner.deref_mut());
    }

    /// Returns the unwrapped pointer or panic. This unwrapping passes the responsibility to the programmer.
    pub fn set_next(&mut self, next: Option<synchronisation::NullLock<AllocPointer>>) {
        self.next = next;
    }
}

impl Allocator {
    /// Initialise the allocation system
    pub fn init() {
    }

    /// Returns the number of addresses marked taken
    pub fn get_alloc_count() -> usize {
        let mut temp_alloc = unsafe { &mut ROOT_ALLOC };
        let mut count: usize = 0;
        while temp_alloc.has_next() {
            if temp_alloc.get_next().get_flags() == AllocFlags::Allocated {
                count += temp_alloc.get_next().get_size();
            }
            temp_alloc = temp_alloc.get_next_mut();
        }
        count
    }

    /// Performs pointer checks and returns the Alloc for it
    pub fn get_ptr_alloc(ptr: *mut u8) -> &'static mut Alloc {
        // Ensure we don't free a null pointer.
        assert!(!ptr.is_null());

        // Make sure that the address makes sense
        unsafe {
            assert!(
                (ptr as usize) >= HEAP_START
                    && (ptr as usize) < HEAP_START + HEAP_SIZE
            );
        }

        // Find an alloc with enough bytes which is marked free
        let mut temp_alloc = unsafe { &mut ROOT_ALLOC };
        while (temp_alloc.get_start_address() != (ptr as usize)) && temp_alloc.has_next() {
            temp_alloc = temp_alloc.get_next_mut();
        }

        // No memory was available
        if temp_alloc.get_start_address() != (ptr as usize) {
            panic!("Received a ptr to an unknown Alloc.")
        }

        temp_alloc
    }
}

/// Allocate a byte or multiple bytes
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // To help prevent data leaks let's zero all memory allocated
        self.alloc_zeroed(layout)
    }

    /// Deallocate a byte by its pointer
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        logln!(
            "DEALLOCATING {:>7} BYTES: {:<10?} -> {:<10?}",
            layout.size(),
            ptr,
            ptr.add(layout.size())
        );

        let temp_alloc = Allocator::get_ptr_alloc(ptr);

        // This if allows the reuse of this function on realloc
        if temp_alloc.get_flags() == AllocFlags::Allocated {
            // Zero the memory addresses
            for address in temp_alloc.get_start_address()..temp_alloc.get_end_address() {
                temp_alloc.set_value(address, 0);
            }
            // Set the Alloc as Free
            temp_alloc.set_flags(AllocFlags::Free);
        }

        // If the next Alloc is free let's amalgamate the space
        if temp_alloc.get_next().get_flags() == AllocFlags::Free {
            logln!("\tAMALGAMATING {:<10?} -> {:<10?} WITH {:<10?} -> {:<10?}",
                temp_alloc.get_start_address(), temp_alloc.get_end_address(),
                temp_alloc.get_next().get_start_address(),
                temp_alloc.get_next().get_end_address()
            );
            let address = temp_alloc.get_start_address();
            temp_alloc.get_next_mut().set_address(address);
            // TODO: DELETE OLD ALLOC
        }

        logln!(
            "DEALLOCATED {:>7} BYTES: {:<10?} -> {:<10?}",
            layout.size(),
            ptr,
            ptr.add(layout.size())
        );
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        assert!(layout.size() > 0);
        logln!("ALLOCATING {:>8} BYTES", layout.size());

        // Find an alloc with enough bytes which is marked free
        let mut temp_alloc = unsafe { &mut ROOT_ALLOC };
        while ((temp_alloc.get_size() < layout.size()) | (temp_alloc.get_flags() != AllocFlags::Free)) && temp_alloc.has_next() {
            temp_alloc = temp_alloc.get_next_mut();
        }

        // No memory was available
        if temp_alloc.get_flags() == AllocFlags::Allocated {
            return null_mut();
        }

        // Calculate the alloc boundary
        let new_end = temp_alloc.get_start_address() + layout.size();

        logln!("\tADDRESS: {:<10?} -> {:<10?}", temp_alloc.get_start_address(), new_end);

        // Set the Alloc as allocated
        temp_alloc.set_flags(AllocFlags::Allocated);

        // Zero the memory addresses
        for address in temp_alloc.get_start_address()..new_end {
            temp_alloc.set_value(address, 0);
        }

        let start_address = temp_alloc.get_start_address();
        if temp_alloc.has_next() {
            let x = temp_alloc.get_next_mut();

            // If the next Alloc is free let's amalgamate the space
            if x.get_flags() == AllocFlags::Free {
                logln!("\tAMALGAMATING {:<10?} -> {:<10?} WITH {:<10?} -> {:<10?}",
                    new_end, start_address,
                    x.get_start_address(),
                    x.get_end_address()
                );
                x.set_address(new_end);
                return temp_alloc.get_start_address() as *mut u8;
            }
            // Else create a new free Alloc between
            else {
                let new_alloc_inner = synchronisation::NullLock::new(AllocInner {addr: new_end, flags: AllocFlags::Free});

                // This is gross we should really just 'let mut new_alloc = Some(x);'
                let next_pointer = Some(synchronisation::NullLock::new(AllocPointer{p: &mut (*(x))}));
                let mut new_alloc = Alloc {curr: new_alloc_inner, next: next_pointer};
                let new_alloc_pointer = Some(synchronisation::NullLock::new(AllocPointer{p: (&mut new_alloc) as *mut Alloc}));
                temp_alloc.set_next(new_alloc_pointer);
                logln!("\tUSING NEW ALLOC FOR {:<10?} -> {:<10?}",
                    new_alloc.get_start_address(), new_alloc.get_end_address()
                );
                return new_alloc.curr.lock(|inner| inner.addr()) as *mut u8;
            }
        }

        // If we don't have an Alloc after create a new one
        let new_alloc_inner = synchronisation::NullLock::new(AllocInner {addr: new_end, flags: AllocFlags::Free});
        let mut new_alloc = Alloc {curr: new_alloc_inner, next: None};
        let new_alloc_pointer = Some(synchronisation::NullLock::new(AllocPointer{p: (&mut new_alloc) as *mut Alloc}));
        temp_alloc.next = new_alloc_pointer;
         logln!("\tUSING NEW ALLOC FOR {:<10?} -> {:<10?}",
            new_alloc.get_start_address(), new_alloc.get_end_address()
        );
        return new_alloc.curr.lock(|inner| inner.addr()) as *mut u8;
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

        let temp_alloc = Allocator::get_ptr_alloc(ptr);

        // Stops dealloc from zeroing
        temp_alloc.set_flags(AllocFlags::Free);
        self.dealloc(ptr, layout);

        // Get a new Alloc
        let new_alloc = Allocator::get_ptr_alloc(self.alloc(layout));

        // Transfer data from old Alloc to new Alloc
        let offset = temp_alloc.get_start_address() - new_alloc.get_start_address();
        for address in temp_alloc.get_start_address()..temp_alloc.get_end_address() {
            new_alloc.set_value(address + offset, temp_alloc.get_value(address));
        }

        // Zero the old memory addresses
        for address in temp_alloc.get_start_address()..temp_alloc.get_end_address() {
            temp_alloc.set_value(address, 0);
        }

        logln!(
            "REALLOCATED {:>7} BYTES: {:<10?} -> {:<10?} to {:<10?} -> {:<10?}",
            layout.size(),
            temp_alloc.get_start_address(), temp_alloc.get_end_address(),
            new_alloc.get_start_address(), new_alloc.get_end_address()
        );

        return new_alloc.get_start_address() as *mut u8;
    }
}

/// If there is an out of memory error, just panic.
#[alloc_error_handler]
fn allocator_error(_layout: Layout) -> ! {
    panic!("Memory allocation failed");
}
