use console::{logln, print};
use alloc::{vec::Vec, format};
use global_allocator::Allocator;

pub fn simple_alloc_check() {
    assert!(Allocator::get_alloc_count() == 0, "{}", format!("Expected no allocations got {}.", Allocator::get_alloc_count()));
    {
        let mut x: Vec<u8> = Vec::new();
        x.push(0);
        assert!(Allocator::get_alloc_count() == 8, "{}", format!("Expected Vector initialisation of 8 bytes got {}.", Allocator::get_alloc_count()));

        for i in 1..8 as u8 {
            x.push(i);
            logln!("ADDRESSES ALLOCATED: {}", Allocator::get_alloc_count());
            logln!("Vector: {:?}", x);
        }

        x.push(8);
        assert!(Allocator::get_alloc_count() == 16, "{}", format!("Expected Vector reallocation of 8 to 16 bytes got {}.", Allocator::get_alloc_count()));

        for i in 9..16 as u8 {
            x.push(i);
        }
        assert!(x == [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], "{}", format!("x did not match expected '[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]' got {:?}.", x));
    }
    assert!(Allocator::get_alloc_count() == 0, "{}", format!("Vector out of scope expected no allocations got {}.", Allocator::get_alloc_count()));
    logln!("ADDRESSES ALLOCATED: {}", Allocator::get_alloc_count());
}

pub fn dual_alloc_check() {
    assert!(Allocator::get_alloc_count() == 0, "{}", format!("Expected no allocations got {}.", Allocator::get_alloc_count()));
    {
        let mut x: Vec<u8> = Vec::new();
        let mut y: Vec<u8> = Vec::new();
        x.push(0);
        y.push(u8::MAX - 0);

        assert!(Allocator::get_alloc_count() == 16, "{}", format!("Expected Vector initialisation of 16 bytes got {}.", Allocator::get_alloc_count()));

        for i in 1..8 as u8 {
            x.push(i);
            y.push(u8::MAX - i);
            logln!("ADDRESSES ALLOCATED: {}", Allocator::get_alloc_count());
            logln!("Vector: {:?}", x);
            logln!("Vector: {:?}", y);
        }

        assert!(x == [0, 1, 2, 3, 4, 5, 6, 7], "{}", format!("x did not match expected '[0, 1, 2, 3, 4, 5, 6, 7]' got {:?}.", x));
        assert!(y == [255, 254, 253, 252, 251, 250, 249, 248], "{}", format!("y did not match expected '[255, 254, 253, 252, 251, 250, 249, 248]' got {:?}.", y));
        // ! ALLOCATING 120 BYTES
        x.push(8);
        logln!("Vector: {:?}", x);
        y.push(u8::MAX - 8);
        logln!("Vector: {:?}", y);
        assert!(Allocator::get_alloc_count() == 32, "{}", format!("Expected Vector reallocation of 16 to 32 bytes got {}.", Allocator::get_alloc_count()));
        assert!(x == [0, 1, 2, 3, 4, 5, 6, 7, 8], "{}", format!("x did not match expected '[0, 1, 2, 3, 4, 5, 6, 7, 8]' got {:?}.", x));
        assert!(y == [255, 254, 253, 252, 251, 250, 249, 248, 247], "{}", format!("y did not match expected '[255, 254, 253, 252, 251, 250, 249, 248, 247]' got {:?}.", y));

        for i in 9..16 as u8 {
            x.push(i);
            y.push(u8::MAX - i);
        }
        assert!(x == [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], "{}", format!("x did not match expected '[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]' got {:?}.", x));
        assert!(y == [255, 254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 244, 243, 242, 241, 240], "{}",
            format!("y did not match expected '[255, 254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 244, 243, 242, 241, 240]' got {:?}.", y));

    }
    assert!(Allocator::get_alloc_count() == 0, "{}", format!("Vector out of scope expected no allocations got {}.", Allocator::get_alloc_count()));
    logln!("ADDRESSES ALLOCATED: {}", Allocator::get_alloc_count());
}

pub fn quad_alloc_check() {
    assert!(Allocator::get_alloc_count() == 0, "{}", format!("Expected no allocations got {}.", Allocator::get_alloc_count()));
    {
        let mut a: Vec<u8> = Vec::new();
        let mut b: Vec<u8> = Vec::new();
        let mut c: Vec<u8> = Vec::new();
        let mut d: Vec<u8> = Vec::new();
        a.push(0 + 0);
        b.push(0 + 16);
        c.push(0 + 32);
        d.push(0 + 48);

        assert!(Allocator::get_alloc_count() == 32, "{}", format!("Expected Vector initialisation of 32 bytes got {}.", Allocator::get_alloc_count()));

        for i in 1..8 as u8 {
            a.push(i + 0);
            b.push(i + 16);
            c.push(i + 32);
            d.push(i + 48);
        }

        for i in 1..8 as u8 {
            assert!(a.as_slice()[i as usize] == (i + 0));
            assert!(b.as_slice()[i as usize] == (i + 16));
            assert!(c.as_slice()[i as usize] == (i + 32));
            assert!(d.as_slice()[i as usize] == (i + 48));
        }

        a.push(8 + 0);
        b.push(8 + 16);
        c.push(8 + 32);
        d.push(8 + 48);
        assert!(Allocator::get_alloc_count() == 64, "{}", format!("Expected Vector reallocation of 32 to 64 bytes got {}.", Allocator::get_alloc_count()));

        for i in 1..9 as u8 {
            assert!(a.as_slice()[i as usize] == (i + 0));
            assert!(b.as_slice()[i as usize] == (i + 16));
            assert!(c.as_slice()[i as usize] == (i + 32));
            assert!(d.as_slice()[i as usize] == (i + 48));
        }

        for i in 9..16 as u8 {
            a.push(i + 0);
            b.push(i + 16);
            c.push(i + 32);
            d.push(i + 48);
        }

        for i in 1..16 as u8 {
            assert!(a.as_slice()[i as usize] == (i + 0));
            assert!(b.as_slice()[i as usize] == (i + 16));
            assert!(c.as_slice()[i as usize] == (i + 32));
            assert!(d.as_slice()[i as usize] == (i + 48));
        }
    }
    assert!(Allocator::get_alloc_count() == 0, "{}", format!("Vector out of scope expected no allocations got {}.", Allocator::get_alloc_count()));
}

pub fn large_alloc_check() {
    assert!(Allocator::get_alloc_count() == 0, "{}", format!("Expected no allocations got {}.", Allocator::get_alloc_count()));
    {
        let mut x: Vec<usize> = Vec::new();

        for i in 0..1000 {
            x.push(i);
        }

        for i in 0..1000 {
            assert!(x.as_slice()[i] == i, "{}", format!("Expected {} got {}.\nx: {:?}", i, x.as_slice()[i], x));
        }
    }
    assert!(Allocator::get_alloc_count() == 0, "{}", format!("Vector out of scope expected no allocations got {}.", Allocator::get_alloc_count()));
}


pub fn suite() {
    print!("SIMPLE ALLOC: ");
    simple_alloc_check();
    print!("PASS\n");

    print!("DUAL ALLOC: ");
    dual_alloc_check();
    print!("PASS\n");

    print!("QUAD ALLOC: ");
    quad_alloc_check();
    print!("PASS\n");

    print!("LARGE ALLOC: ");
    large_alloc_check();
    print!("PASS\n");
}
