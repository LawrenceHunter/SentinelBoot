//! Global Allocator
#![no_std]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![feature(strict_provenance)]
#![feature(allocator_api)]
#![feature(ptr_from_ref)]

use core::mem::size_of;
use core::alloc::*;
use console::logln;

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
static HEAP_START: usize = 0x00000000800830c8;

static ALLOC_HEAP_SIZE: usize = 0x1000;
static mut CURR_ALLOC_OFFSET: usize = 0x0;

static HEAP_PUBLIC_START: usize = HEAP_START + ALLOC_HEAP_SIZE;
static HEAP_SIZE: usize =  0x7f7cf38  - ALLOC_HEAP_SIZE;

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

/// Every byte is described by the Alloc structure forming a linked list
pub struct Alloc {
    flag: AllocFlags,
    curr: usize,
    prev: Option<usize>,
    next: Option<usize>,
}

impl From<usize> for Alloc {
    fn from(value: usize) -> Self {
        unsafe {
            let ptr = core::ptr::from_exposed_addr_mut(value) as *mut Alloc;
            return ptr.read();
        };
    }
}

impl Alloc {
    /// Constructor which handles allocation
    pub fn new(flag: AllocFlags, curr: usize, prev: Option<usize>, next: Option<usize>) -> Self {
        let mut new_alloc: Alloc;

        unsafe {
            assert!(CURR_ALLOC_OFFSET < ALLOC_HEAP_SIZE);
            let new_alloc_ptr = core::ptr::from_exposed_addr_mut(HEAP_START + CURR_ALLOC_OFFSET) as *mut Alloc;

            CURR_ALLOC_OFFSET += size_of::<Alloc>();

            new_alloc = new_alloc_ptr.read();
        };

        new_alloc.set_flag(flag);
        new_alloc.set_start_address(curr);
        new_alloc.set_prev(prev);
        new_alloc.set_next(next);

        new_alloc
    }

    /// Returns the AllocFlags
    pub fn get_flag(&self) -> AllocFlags {
        self.flag
    }

    /// Sets the flags of the Alloc
    pub fn set_flag(&mut self, flag: AllocFlags) {
        self.flag = flag;
    }

    /// Returns the address pointer
    pub fn get_start_address(&self) -> usize {
        self.curr
    }

    /// Sets the current address
    pub fn set_start_address(&mut self, curr: usize) {
        self.curr = curr;
    }

    /// Returns the pointer to the previous Alloc
    pub fn get_prev(&self) -> Option<usize> {
        self.prev
    }

    /// Sets the pointer for the previous Alloc
    pub fn set_prev(&mut self, prev: Option<usize>) {
        self.prev = prev;
    }

    /// Returns the pointer to the next Alloc
    pub fn get_next(&self) -> Option<usize> {
        self.next
    }

    /// Sets the pointer for the next Alloc
    pub fn set_next(&mut self, next: Option<usize>) {
        self.next = next;
    }

    /// Returns the final address the Alloc controls
    pub fn get_end_address(&self) -> usize {
        if self.get_next().is_none() {
            return HEAP_PUBLIC_START + HEAP_SIZE;
        }

        let next = Alloc::from(self.get_next().unwrap());
        next.get_start_address() - self.get_start_address()
    }

    /// Sets the value of a byte in the Alloc's control
    pub fn set_value(&self, address: usize, value: u8) {
        assert!(address >= self.get_start_address());
        assert!(address < self.get_end_address());
        unsafe { core::ptr::write(address as *mut u8, value) };
    }

    /// Gets the value of a byte in the Alloc's control
    pub fn get_value(&self, address: usize) -> u8 {
        assert!(address >= self.get_start_address());
        assert!(address < self.get_end_address());
        unsafe { return core::ptr::read(address as *mut u8) };
    }

    /// Returns the Alloc of next
    pub fn get_next_deref(&self) -> Alloc {
        assert!(self.get_next().is_some());
        Alloc::from(self.get_next().unwrap())
    }

    /// Returns the Alloc of prev
    pub fn get_prev_deref(&self) -> Alloc {
        assert!(self.get_next().is_some());
        Alloc::from(self.get_next().unwrap())
    }

    /// Gets the number of addresses the Alloc controls
    pub fn get_size(&self) -> usize {
        self.get_end_address() - self.get_start_address()
    }
}

impl Allocator {
    /// Initialise the allocation system
    pub fn init() {

    }


     /// Returns the number of addresses marked taken
     pub fn get_alloc_count() -> usize {
        let mut temp_alloc = Allocator::get_ptr_alloc(HEAP_PUBLIC_START as *mut u8);
        let mut count: usize = 0;
        while temp_alloc.get_next().is_some() {
            if temp_alloc.get_next_deref().get_flag() == AllocFlags::Allocated {
                count += temp_alloc.get_next_deref().get_size();
            }
            temp_alloc = temp_alloc.get_next_deref();
        }
        count
    }

    /// Performs pointer checks and returns the Alloc for it
    pub fn get_ptr_alloc(ptr: *mut u8) -> Alloc {
        // Ensure we don't free a null pointer.
        assert!(!ptr.is_null());

        // Make sure that the address makes sense
        // unsafe {
        //     assert!(
        //         (ptr as usize) >= HEAP_START
        //             && (ptr as usize) < HEAP_START + HEAP_SIZE
        //     );
        // }
        assert!(
            (ptr as usize) >= HEAP_START
                && (ptr as usize) < HEAP_START + HEAP_SIZE
        );

        // Find an alloc with enough bytes which is marked free
        let mut temp_alloc = Allocator::get_ptr_alloc(HEAP_PUBLIC_START as *mut u8);
        while (temp_alloc.get_start_address() != (ptr as usize)) && temp_alloc.get_next().is_some() {
            temp_alloc = temp_alloc.get_next_deref();
        }

        // No memory was available
        if temp_alloc.get_start_address() != (ptr as usize) {
            panic!("Received a ptr to an unknown Alloc.")
        }

        temp_alloc
    }
}

/// Embedded implementation for heap memory allocation
#[derive(Default)]
pub struct Allocator;

/// The static global allocator.
#[global_allocator]
static mut GLOBAL_ALLOCATOR: Allocator = Allocator;

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

        let mut temp_alloc = Allocator::get_ptr_alloc(HEAP_PUBLIC_START as *mut u8);

        // This if allows the reuse of this function on realloc
        if temp_alloc.get_flag() == AllocFlags::Allocated {
            // Zero the memory addresses
            for address in temp_alloc.get_start_address()..temp_alloc.get_end_address() {
                temp_alloc.set_value(address, 0);
            }
            // Set the Alloc as Free
            temp_alloc.set_flag(AllocFlags::Free);
        }

        // If the next Alloc is free let's amalgamate the space
        if temp_alloc.get_next_deref().get_flag() == AllocFlags::Free {
            logln!("\tAMALGAMATING {:<10?} -> {:<10?} WITH {:<10?} -> {:<10?}",
                temp_alloc.get_start_address(), temp_alloc.get_end_address(),
                temp_alloc.get_next_deref().get_start_address(),
                temp_alloc.get_next_deref().get_end_address()
            );
            let address = temp_alloc.get_start_address();
            temp_alloc.get_next_deref().set_start_address(address);
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
        let mut temp_alloc = Allocator::get_ptr_alloc(HEAP_PUBLIC_START as *mut u8);

        while ((temp_alloc.get_size() < layout.size()) | (temp_alloc.get_flag() != AllocFlags::Free)) && temp_alloc.get_next().is_some() {
            temp_alloc = temp_alloc.get_next_deref();
        }

        // No memory was available
        if temp_alloc.get_flag() == AllocFlags::Allocated {
            return core::ptr::null_mut();
        }

        // Calculate the alloc boundary
        let new_end = temp_alloc.get_start_address() + layout.size();

        logln!("\tADDRESS: {:<10?} -> {:<10?}", temp_alloc.get_start_address(), new_end);

        // Set the Alloc as allocated
        temp_alloc.set_flag(AllocFlags::Allocated);

        // Zero the memory addresses
        for address in temp_alloc.get_start_address()..new_end {
            temp_alloc.set_value(address, 0);
        }

        let start_address = temp_alloc.get_start_address();
        if temp_alloc.get_next().is_some() {
            let mut x = temp_alloc.get_next_deref();

            // If the next Alloc is free let's amalgamate the space
            if x.get_flag() == AllocFlags::Free {
                logln!("\tAMALGAMATING {:<10?} -> {:<10?} WITH {:<10?} -> {:<10?}",
                    new_end, start_address,
                    x.get_start_address(),
                    x.get_end_address()
                );
                x.set_start_address(new_end);
                return temp_alloc.get_start_address() as *mut u8;
            }
            // Else create a new free Alloc between
            else {

                // Old implementation keeping for now for reference
                // let new_alloc_inner = synchronisation::NullLock::new(AllocInner {addr: new_end, flags: AllocFlags::Free});
                // // This is gross we should really just 'let mut new_alloc = Some(x);'
                // let next_pointer = Some(synchronisation::NullLock::new(AllocPointer{p: &mut (*(x))}));
                // let mut new_alloc = Alloc {curr: new_alloc_inner, next: next_pointer};
                // let new_alloc_pointer = Some(synchronisation::NullLock::new(AllocPointer{p: (&mut new_alloc) as *mut Alloc}));
                // temp_alloc.set_next(new_alloc_pointer);
                // logln!("\tUSING NEW ALLOC FOR {:<10?} -> {:<10?}",
                //     new_alloc.get_start_address(), new_alloc.get_end_address()
                // );
                // return new_alloc.curr.lock(|inner| inner.addr()) as *mut u8;

                let new_alloc = Alloc::new(AllocFlags::Free, new_end, Some(core::ptr::from_mut(&mut temp_alloc) as usize), Some(core::ptr::from_mut(&mut x) as usize));
                return new_alloc.get_start_address() as *mut u8;
            }
        }

        // Old implementation keeping for now for reference
        // // If we don't have an Alloc after create a new one
        // let new_alloc_inner = synchronisation::NullLock::new(AllocInner {addr: new_end, flags: AllocFlags::Free});
        // let mut new_alloc = Alloc {curr: new_alloc_inner, next: None};
        // let new_alloc_pointer = Some(synchronisation::NullLock::new(AllocPointer{p: (&mut new_alloc) as *mut Alloc}));
        // temp_alloc.next = new_alloc_pointer;
        //  logln!("\tUSING NEW ALLOC FOR {:<10?} -> {:<10?}",
        //     new_alloc.get_start_address(), new_alloc.get_end_address()
        // );
        // return new_alloc.curr.lock(|inner| inner.addr()) as *mut u8;
        let mut new_alloc = Alloc::new(AllocFlags::Free, new_end, Some(core::ptr::from_mut(&mut temp_alloc) as usize), None);
        temp_alloc.set_next(Some(core::ptr::from_mut(&mut new_alloc) as usize));
        return new_alloc.get_start_address() as *mut u8;
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        assert!(new_size > 0);
        logln!(
            "REALLOCATING {:?} FROM {} BYTES TO {} BYTES",
            ptr,
            layout.size(),
            new_size
        );

        let mut temp_alloc = Allocator::get_ptr_alloc(HEAP_PUBLIC_START as *mut u8);

        // Stops dealloc from zeroing
        temp_alloc.set_flag(AllocFlags::Free);
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
