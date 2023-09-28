//! Global Allocator
#![no_std]
#![feature(format_args_nl)]
#![feature(alloc_error_handler)]
#![feature(strict_provenance)]
#![feature(allocator_api)]
#![feature(ptr_from_ref)]

use core::fmt::Display;
use core::alloc::*;
use core::mem::size_of;
// use console::logln;

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
const HEAP_START: usize = 0x0000000081000000;

static ALLOC_HEAP_SIZE: usize = 0x100000;
static mut CURR_ALLOC_OFFSET: usize = 0x0;

static HEAP_PUBLIC_START: usize = HEAP_START + ALLOC_HEAP_SIZE;
static HEAP_SIZE: usize =  0x7f7cf38  - ALLOC_HEAP_SIZE;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Represents if the Alloc is free or allocated
#[repr(u8)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AllocFlags {
    /// Currently in use
    Allocated = 0,
    /// Available
    Free = 1,
    /// Ignore me
    Dead = 2,
}

/// Every byte is described by the Alloc structure forming a linked list
#[derive(Debug)]
/// Forces struct ordering
#[repr(C)]
pub struct Alloc {
    flag: AllocFlags,
    curr: usize,
    prev: Option<usize>,
    next: Option<usize>,
}

impl Display for Alloc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe { return write!(f, "\n| Allocated:     {:?}\n| Start Address: {:#018x}\n| End Address:   {:#018x}\n| \
         Previous:      {:#018x} ({:#018x} -> {:#018x})\n| Next:          {:#018x} ({:#018x} -> {:#018x})\n| Size:          {}",
            self.get_flag(), self.get_start_address(), self.get_end_address(),
            if self.get_prev().is_some() {self.get_prev().unwrap()} else {usize::MAX},
            if self.get_prev().is_some() {(*(self.get_prev().unwrap() as *mut Alloc)).get_start_address()} else {usize::MAX},
            if self.get_prev().is_some() {(*(self.get_prev().unwrap() as *mut Alloc)).get_end_address()} else {usize::MAX},
            if self.get_next().is_some() {self.get_next().unwrap()} else {usize::MAX},
            if self.get_next().is_some() {(*(self.get_next().unwrap() as *mut Alloc)).get_start_address()} else {usize::MAX},
            if self.get_next().is_some() {(*(self.get_next().unwrap() as *mut Alloc)).get_end_address()} else {usize::MAX},
            self.get_size());
        };
    }
}

impl Alloc {
    /// Constructor which handles allocation
    pub unsafe fn new(flag: AllocFlags, curr: usize, prev: Option<usize>, next: Option<usize>) -> *mut Alloc {
        let new_alloc_ptr: *mut Alloc;

        unsafe {
            assert!(CURR_ALLOC_OFFSET < ALLOC_HEAP_SIZE);
            // logln!("(new) CURR_ALLOC_OFFSET: {:#018x}", CURR_ALLOC_OFFSET);
            new_alloc_ptr = (HEAP_START + CURR_ALLOC_OFFSET) as *mut Alloc;

            CURR_ALLOC_OFFSET += size_of::<Alloc>();
            // CURR_ALLOC_OFFSET += 0x100;

            (*(new_alloc_ptr)).set_flag(flag);
            (*(new_alloc_ptr)).set_start_address(curr);
            (*(new_alloc_ptr)).set_prev(prev);
            (*(new_alloc_ptr)).set_next(next);

            // logln!("(new) CREATED ALLOC:     {}", new_alloc_ptr.as_ref().unwrap_unchecked());
            // logln!("(new) CURR_ALLOC_OFFSET: {:#08x}", CURR_ALLOC_OFFSET);
            // logln!("(new) ALLOC ADDR:        {:?}\n", new_alloc_ptr);
            // logln!("(new) ALLOC FLAG ADDR:   {:?}", core::ptr::addr_of!((*(new_alloc_ptr)).flag));
            // logln!("(new) ALLOC START ADDR:  {:?}", core::ptr::addr_of!((*(new_alloc_ptr)).curr));
            // logln!("(new) ALLOC PREV ADDR:   {:?}", core::ptr::addr_of!((*(new_alloc_ptr)).prev));
            // logln!("(new) ALLOC NEXT ADDR:   {:?}", core::ptr::addr_of!((*(new_alloc_ptr)).next));
        };

        new_alloc_ptr
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

        let next = self.get_next_deref();
        // logln!("(get_end_address) CURR: {}", self);
        // unsafe { logln!("(get_end_address) NEXT: {}", (*(next))) };
        // unsafe { logln!("(get_end_address) SIZE: {:#018x}\t START: {:#018x}\t END: {:#018x}", (*(next)).get_start_address() - self.get_start_address(), self.get_start_address(), (*(next)).get_start_address()) };
        unsafe { return (*(next)).get_start_address(); }
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
    pub fn get_next_deref(&self) -> *mut Alloc {
        assert!(self.get_next().is_some());
        self.get_next().unwrap() as *mut Alloc
    }

    /// Returns the Alloc of prev
    pub fn get_prev_deref(&self) -> *mut Alloc {
        assert!(self.get_next().is_some());
        self.get_prev().unwrap() as *mut Alloc
    }

    /// Gets the number of addresses the Alloc controls
    pub fn get_size(&self) -> usize {
        // logln!("(get_size) SIZE: {:#018x}\t END: {:#018x}\t START:{:#018x}", self.get_end_address() - self.get_start_address(), self.get_end_address(), self.get_start_address());
        self.get_end_address() - self.get_start_address()
    }
}

impl Allocator {
    /// Initialise the allocation system
    pub fn init() {
    //     logln!("(init) HEAP START:        {:#018x}", HEAP_START);
    //     logln!("(init) HEAP PUBLIC START: {:#018x}", HEAP_PUBLIC_START);
    //     logln!("(init) HEAP SIZE ALLOC:   {:#018x}", ALLOC_HEAP_SIZE);
    //     logln!("(init) HEAP SIZE PUBLIC:  {:#018x}", HEAP_SIZE);

        unsafe { Alloc::new(AllocFlags::Free, HEAP_PUBLIC_START, None, None) };

        // logln!("(init) HEAP START:        {:#018x}", HEAP_START);
        // let temp_alloc = HEAP_START as *mut Alloc;
        // unsafe {
            // logln!("(init) GOT ALLOC:         {}", (*(temp_alloc)));
            // logln!("(init) ALLOC ADDR:        {:?}\n", temp_alloc);
            // logln!("(init) ALLOC FLAG ADDR:   {:?}", core::ptr::addr_of!((*(temp_alloc)).flag));
            // logln!("(init) ALLOC START ADDR:  {:?}", core::ptr::addr_of!((*(temp_alloc)).curr));
            // logln!("(init) ALLOC PREV ADDR:   {:?}", core::ptr::addr_of!((*(temp_alloc)).prev));
            // logln!("(init) ALLOC NEXT ADDR:   {:?}", core::ptr::addr_of!((*(temp_alloc)).next));
        // }
    }


     /// Returns the number of addresses marked taken
     pub fn get_alloc_count() -> usize {
        let mut temp_alloc = Allocator::get_ptr_alloc(HEAP_PUBLIC_START as *mut u8);
        let mut count: usize = 0;
        unsafe {
            while (*(temp_alloc)).get_next().is_some() {
                // logln!("ALLOCATED: {:?}\nCOUNT: {}", (*(temp_alloc)).get_flag(), count);
                if (*(temp_alloc)).get_flag() == AllocFlags::Allocated {
                    count += (*(temp_alloc)).get_size();
                }
                // logln!("ALLOCATED: {:?}\nCOUNT: {}", (*(temp_alloc)).get_flag(), count);
                temp_alloc = (*(temp_alloc)).get_next_deref();
            }
        }
        count
    }

    /// Performs pointer checks and returns the Alloc for it
    pub fn get_ptr_alloc(ptr: *mut u8) -> *mut Alloc {
        // Ensure we don't free a null pointer.
        assert!(!ptr.is_null());

        // logln!("(get_ptr_alloc) FINDING ALLOC FOR {:#018x}", ptr as usize);

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

        // Need to reason about this line but works for now
        let mut temp_alloc = HEAP_START as *mut Alloc;
        unsafe {
            // logln!("(get_ptr_alloc) GOT ALLOC: {:?}", core::ptr::read(temp_alloc));
            // logln!("(get_ptr_alloc) ALLOC ADDR: {:?}", temp_alloc);
            // logln!("ALLOC FLAG ADDR:  {:?}", core::ptr::addr_of!((*(temp_alloc)).flag));
            // logln!("ALLOC START ADDR: {:?}", core::ptr::addr_of!((*(temp_alloc)).curr));
            // logln!("ALLOC PREV ADDR:  {:?}", core::ptr::addr_of!((*(temp_alloc)).prev));
            // logln!("ALLOC NEXT ADDR:  {:?}", core::ptr::addr_of!((*(temp_alloc)).next));

            // TODO: Allow reuse of space
            while (((*(temp_alloc)).get_start_address() != (ptr as usize)) || (*(temp_alloc)).get_flag() == AllocFlags::Dead) && (*(temp_alloc)).get_next().is_some() {
                temp_alloc = (*(temp_alloc)).get_next_deref();
                // logln!("(get_ptr_alloc) GOT ALLOC: {:?}", core::ptr::read(temp_alloc));
                // logln!("(get_ptr_alloc) ALLOC ADDR: {:?}", temp_alloc);
                // logln!("ALLOC FLAG ADDR:  {:?}", core::ptr::addr_of!((*(temp_alloc)).flag));
                // logln!("ALLOC START ADDR: {:?}", core::ptr::addr_of!((*(temp_alloc)).curr));
                // logln!("ALLOC PREV ADDR:  {:?}", core::ptr::addr_of!((*(temp_alloc)).prev));
                // logln!("ALLOC NEXT ADDR:  {:?}", core::ptr::addr_of!((*(temp_alloc)).next));
            }

            // No memory was available
            if (*(temp_alloc)).get_start_address() != (ptr as usize) {
                panic!("Received a ptr to an unknown Alloc.")
            }

            // logln!("(get_ptr_alloc) GOT ALLOC: {}", (*(temp_alloc)));
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
        assert!(layout.size() > 0);
        // logln!("(alloc) ALLOCATING {} BYTES", layout.size());

        // Find an alloc with enough bytes which is marked free
        let mut temp_alloc = Allocator::get_ptr_alloc(HEAP_PUBLIC_START as *mut u8);

        while ((*(temp_alloc)).get_size() < layout.size()) | (((*(temp_alloc)).get_flag() != AllocFlags::Free)) && ((*(temp_alloc)).get_next().is_some()) {
            temp_alloc = ((*(temp_alloc))).get_next_deref();
        }

        // logln!("(alloc) GOT ALLOC: {}", (*(temp_alloc)));

        // No memory was available
        if (*(temp_alloc)).get_flag() == AllocFlags::Allocated {
            return core::ptr::null_mut();
        }

        // Calculate the alloc boundary
        let new_end = (*(temp_alloc)).get_start_address() + layout.size();

        // logln!("(alloc) ADDRESS: {:#018x} -> {:#018x}", (*(temp_alloc)).get_start_address(), new_end);

        // Set the Alloc as allocated
        (*(temp_alloc)).set_flag(AllocFlags::Allocated);

        if (*(temp_alloc)).get_next().is_some() {
            // logln!("CHECKING IF AMALGAMATION POSSIBLE");
            let x = (*(temp_alloc)).get_next_deref();

            // If the next Alloc is free let's amalgamate the space
            if (*(x)).get_flag() == AllocFlags::Free {
                // logln!("(alloc) AMALGAMATING {:#018x} -> {:#018x} WITH {:#018x} -> {:#018x}",
                //     new_end, (*(temp_alloc)).get_start_address(),
                //     (*(x)).get_start_address(),
                //     (*(x)).get_end_address()
                // );
                (*(x)).set_start_address(new_end);
                // logln!("(alloc) GOT ALLOC: {}", (*(x)));
                return (*(temp_alloc)).get_start_address() as *mut u8;
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

                // logln!("(alloc) CREATING ALLOC {:#018x} -> {:#018x}", (*(temp_alloc)).get_start_address(), new_end);
                let new_alloc = Alloc::new(AllocFlags::Free, new_end, Some(temp_alloc as usize), Some(x as usize));
                (*(temp_alloc)).set_next(Some(new_alloc as usize));
                (*(x)).set_prev(Some(new_alloc as usize));
                // logln!("(alloc) NEW ALLOC {}", (*(new_alloc)));
                return (*(temp_alloc)).get_start_address() as *mut u8;
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
        // logln!("(alloc) CREATING ALLOC {:#018x} -> {:#018x}", start_address, new_end);
        let new_alloc = Alloc::new(AllocFlags::Free, new_end, Some(temp_alloc as usize), None);
        (*(temp_alloc)).set_next(Some(new_alloc as usize));
        // logln!("(alloc) NEW ALLOC {}", (*(new_alloc)));
        // logln!("(alloc) OLD ALLOC {}", (*(temp_alloc)));
        return (*(temp_alloc)).get_start_address() as *mut u8;
    }

    /// Deallocate a byte by its pointer
    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        // logln!(
        //     "(dealloc) DEALLOCATING {} BYTES: {:#018x} -> {:#018x}",
        //     layout.size(),
        //     ptr as usize,
        //     ptr.add(layout.size()) as usize
        // );

        let temp_alloc = Allocator::get_ptr_alloc(ptr);

        // logln!("(dealloc) GOT ALLOC: {}", (*(temp_alloc)));

        // This if allows the reuse of this function on realloc
        if (*(temp_alloc)).get_flag() == AllocFlags::Allocated {
            // Zero the memory addresses
            for address in (*(temp_alloc)).get_start_address()..(*(temp_alloc)).get_end_address() {
                (*(temp_alloc)).set_value(address, 0);
            }
            // Set the Alloc as Free
            (*(temp_alloc)).set_flag(AllocFlags::Free);
        }

        // If the next Alloc is free let's amalgamate the space
        if (*((*(temp_alloc)).get_next_deref())).get_flag() == AllocFlags::Free {
            // logln!("(dealloc) AMALGAMATING {:#018x} -> {:#018x} WITH {:#018x} -> {:#018x}",
            //     (*(temp_alloc)).get_start_address(), (*(temp_alloc)).get_end_address(),
            //     (*((*(temp_alloc)).get_next_deref())).get_start_address(),
            //     (*((*(temp_alloc)).get_next_deref())).get_end_address()
            // );
            let address = (*(temp_alloc)).get_start_address();
            (*((*(temp_alloc)).get_next_deref())).set_start_address(address);
            (*((*(temp_alloc)).get_next_deref())).set_prev((*(temp_alloc)).get_prev());
            // TODO: Make more efficient so we can reuse the space
            // logln!("(dealloc) AMAL ALLOC: {}", (*((*(temp_alloc)).get_next_deref())));
            (*(temp_alloc)).set_flag(AllocFlags::Dead);
            // TODO: DELETE OLD ALLOC
        }

        // logln!(
        //     "DEALLOCATED {} BYTES: {:#018x} -> {:#018x}",
        //     layout.size(),
        //     ptr as usize,
        //     ptr.add(layout.size()) as usize
        // );
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        assert!(layout.size() > 0);
        // logln!("(alloc_zeroed) ALLOCATING {} BYTES", layout.size());

        // Find an alloc with enough bytes which is marked free
        let mut temp_alloc = Allocator::get_ptr_alloc(HEAP_PUBLIC_START as *mut u8);

        while ((*(temp_alloc)).get_size() < layout.size()) | (((*(temp_alloc)).get_flag() != AllocFlags::Free)) && ((*(temp_alloc)).get_next().is_some()) {
            temp_alloc = ((*(temp_alloc))).get_next_deref();
        }

        // logln!("(alloc_zeroed) GOT ALLOC: {}", (*(temp_alloc)));

        // No memory was available
        if (*(temp_alloc)).get_flag() == AllocFlags::Allocated {
            return core::ptr::null_mut();
        }

        // Calculate the alloc boundary
        let new_end = (*(temp_alloc)).get_start_address() + layout.size();

        // logln!("(alloc_zeroed) ADDRESS: {:#018x} -> {:#018x}", (*(temp_alloc)).get_start_address(), new_end);

        // Set the Alloc as allocated
        (*(temp_alloc)).set_flag(AllocFlags::Allocated);

        // Zero the memory addresses
        for address in (*(temp_alloc)).get_start_address()..new_end {
            (*(temp_alloc)).set_value(address, 0);
            // logln!("(alloc_zeroed) ZEROING {:#018x}", address);
        }

        if (*(temp_alloc)).get_next().is_some() {
            // logln!("CHECKING IF AMALGAMATION POSSIBLE");
            let x = (*(temp_alloc)).get_next_deref();

            // If the next Alloc is free let's amalgamate the space
            if (*(x)).get_flag() == AllocFlags::Free {
                // logln!("(alloc_zeroed) AMALGAMATING {:#018x} -> {:#018x} WITH {:#018x} -> {:#018x}",
                //     new_end, (*(temp_alloc)).get_start_address(),
                //     (*(x)).get_start_address(),
                //     (*(x)).get_end_address()
                // );
                (*(x)).set_start_address(new_end);
                // logln!("(alloc_zeroed) GOT ALLOC: {}", (*(x)));
                return (*(temp_alloc)).get_start_address() as *mut u8;
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

                // logln!("(alloc_zeroed) CREATING ALLOC {:#018x} -> {:#018x}", (*(temp_alloc)).get_start_address(), new_end);
                let new_alloc = Alloc::new(AllocFlags::Free, new_end, Some(temp_alloc as usize), Some(x as usize));
                (*(temp_alloc)).set_next(Some(new_alloc as usize));
                (*(x)).set_prev(Some(new_alloc as usize));
                // logln!("(alloc_zeroed) NEW ALLOC {}", (*(new_alloc)));
                return (*(temp_alloc)).get_start_address() as *mut u8;
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
        // logln!("(alloc_zeroed) CREATING ALLOC {:#018x} -> {:#018x}", start_address, new_end);
        let new_alloc = Alloc::new(AllocFlags::Free, new_end, Some(temp_alloc as usize), None);
        (*(temp_alloc)).set_next(Some(new_alloc as usize));
        // logln!("(alloc_zeroed) NEW ALLOC {}", (*(new_alloc)));
        // logln!("(alloc_zeroed) OLD ALLOC {}", (*(temp_alloc)));
        return (*(temp_alloc)).get_start_address() as *mut u8;
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        assert!(new_size > 0);

        // logln!(
        //     "(realloc) REALLOCATING {:#018x} FROM {} BYTES TO {} BYTES",
        //     ptr as usize,
        //     layout.size(),
        //     new_size
        // );

        let temp_alloc = Allocator::get_ptr_alloc(HEAP_PUBLIC_START as *mut u8);
        let old_ptr = (*(temp_alloc)).get_start_address();

        // logln!("(realloc) DEALLOCATING {}", (*(temp_alloc)));

        // Stops dealloc from zeroing
        (*(temp_alloc)).set_flag(AllocFlags::Free);
        self.dealloc(ptr, layout);
        // logln!("(realloc) DEALLOCATED");

        // Get a new Alloc
        let new_layout = core::alloc::Layout::from_size_align(new_size, layout.align()).unwrap();
        let new_alloc = Allocator::get_ptr_alloc(self.alloc(new_layout));
        // logln!("(realloc) NEW ALLOC {}", (*(new_alloc)));

        // Transfer data from old Alloc to new Alloc
        let offset = old_ptr - (*(new_alloc)).get_start_address();
        // logln!("(realloc) OFFSET {:#018x}", offset);
        // logln!("(realloc) RANGE {:#018x} -> {:#018x}", old_ptr, old_ptr + layout.size());
        for address in old_ptr..old_ptr + layout.size() {
            (*(new_alloc)).set_value(address + offset, core::ptr::read(address as *mut u8));
            // logln!("(realloc) COPYING {:#018x} ({:#02x})-> {:#018x}", address, core::ptr::read(address as *mut u8), address + offset);
        }

        // Zero the old memory addresses
        for address in (*(temp_alloc)).get_start_address()..(*(temp_alloc)).get_end_address() {
            core::ptr::write_bytes(address as *mut u8, 0, 1);
            // logln!("(realloc) ZEROING {:#018x}", address);
        }

        // logln!(
        //     "(realloc) REALLOCATED {} BYTES: {:#018x} -> {:#018x} to {:#018x} -> {:#018x}",
        //     layout.size(),
        //     old_ptr, old_ptr,
        //     (*(new_alloc)).get_start_address(), (*(new_alloc)).get_end_address()
        // );

        return (*(new_alloc)).get_start_address() as *mut u8;
    }
}

/// If there is an out of memory error, just panic.
#[alloc_error_handler]
fn allocator_error(_layout: Layout) -> ! {
    panic!("Memory allocation failed");
}
