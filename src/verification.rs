use console::{print, println};
use core::slice;
use pelite::pe64::{self, Pe};
// #[cfg(not(feature = "qemu"))]
use sha2::{Digest, Sha256};

pub fn verify_kernel() -> Result<(), ed25519_compact::Error> {
    println!("Hashing stored kernel...");

    let hash = hash_kernel();

    println!("Stored kernel hashed:");
    pretty_print_slice(hash.as_slice());

    println!("Loading server public key...");
    let public_key =
        ed25519_compact::PublicKey::from_slice(crate::helper::PUBLIC_KEY)
            .unwrap();
    println!("Loaded server public key:");
    pretty_print_slice(public_key.as_slice());

    println!("Loading kernel signature...");
    let signature_bytes = unsafe {
        slice::from_raw_parts(
            (bsp::memory::map::kernel::SIGNATURE) as *mut u8,
            64,
        )
    };
    let signature =
        ed25519_compact::Signature::from_slice(signature_bytes).unwrap();
    println!("Loaded kernel signature:");
    pretty_print_slice(signature.as_slice());

    println!("Verifying stored kernel...");
    public_key.verify(hash.as_slice(), &signature)
}

fn get_kernel_size() -> usize {
    println!("Determining kernel size...");
    let data = unsafe {
        slice::from_raw_parts(
            (bsp::memory::map::kernel::KERNEL) as *mut u8,
            bsp::memory::map::kernel::DTB - bsp::memory::map::kernel::KERNEL,
        )
    };
    let pe = pe64::PeFile::from_bytes(data).unwrap();
    let kernel_size: usize = pe.optional_header().AddressOfEntryPoint as usize;
    println!("Kernel size: 0x{:X?}", kernel_size);
    64 * (kernel_size / 64) // normalises to multiple of 64
}

#[cfg(not(feature = "qemu"))]
fn hash_kernel() -> [u8; 32] {
    let mut hasher = Sha256::new();
    let mut offset = 0;
    let buff_size = 4096;
    let kernel_size = get_kernel_size();
    println!(
        "Kernel range: 0x{:X?} -> 0x{:X?}",
        bsp::memory::map::kernel::KERNEL,
        bsp::memory::map::kernel::KERNEL + kernel_size
    );
    loop {
        let data = unsafe {
            slice::from_raw_parts(
                (bsp::memory::map::kernel::KERNEL + (offset * buff_size))
                    as *mut u8,
                buff_size,
            )
        };
        if (offset * buff_size) >= kernel_size {
            break;
        }
        hasher.update(data);
        offset += 1;
    }
    hasher.finalize().into()
}

fn hash_kernel_serial() -> [u8; 32] {
    let mut hasher = Sha256::new();
    let mut offset = 0;
    let buff_size = 4096;
    let kernel_size = get_kernel_size();
    println!(
        "Kernel range: 0x{:X?} -> 0x{:X?}",
        bsp::memory::map::kernel::KERNEL,
        bsp::memory::map::kernel::KERNEL + kernel_size
    );
    loop {
        let data = unsafe {
            slice::from_raw_parts(
                (bsp::memory::map::kernel::KERNEL + (offset * buff_size))
                    as *mut u8,
                buff_size,
            )
        };
        if (offset * buff_size) >= kernel_size {
            break;
        }
        hasher.update(data);
        offset += 1;
    }
    hasher.finalize().into()
}


#[cfg(feature = "qemu")]
fn hash_kernel() -> [u8; 32] {
    use core::arch::asm;

    let mut kernel_size = get_kernel_size();
    println!(
        "Kernel range: 0x{:X?} -> 0x{:X?}",
        bsp::memory::map::kernel::KERNEL,
        bsp::memory::map::kernel::KERNEL + kernel_size
    );
    let mut result: [u64; 4] = [0, 0, 0, 0];
    result[0] = unsafe { core::ptr::read(bsp::memory::map::kernel::KERNEL as *const u64) };
    result[1] = unsafe { core::ptr::read((bsp::memory::map::kernel::KERNEL + 8) as *const u64) };
    result[2] = unsafe { core::ptr::read((bsp::memory::map::kernel::KERNEL + 16) as *const u64) };
    result[3] = unsafe { core::ptr::read((bsp::memory::map::kernel::KERNEL + 24) as *const u64) };
    let mut loop_count = 1;
    // kernel_size = 256;
    // println!("Attempting enabling mstatus.vs");
    // unsafe {
    //     asm!(
    //         "csrr t0, mstatus",
    //         "ori t0, t0, 0x200",
    //         "csrrw x0, mstatus, t0",
    //         out("t0") _,
    //     )
    // }
    println!("Attempting vector hashing - kernel size: {}B", kernel_size);
    println!("\tResult: {:?}", result);
    while kernel_size >= 32 {
        let kernel_pointer = bsp::memory::map::kernel::KERNEL + (loop_count * 64);
        unsafe {
            asm!(
                "li a0, 8",
                "addi a2, a2, 1",
                // ".word 0x01a57a77", // vsetvli t0, a0, e32, m1, tu, mu
                ".word 0x010572d7", // vsetvli t0, a0, e32, m1, tu, mu
                ".word 0x0206f007", // vle64.v v0, (a3)
                ".word 0x0205f207", // vle64.v v4, (a1)
                "add a1, a1, 32",   // Sanity check
                ".word 0x0205f407", // vle64.v v8, (a1)
                ".word 0xB6822077", // vsha2ms.vv v0, v8, v4
                ".word 0x0206f027", // vse64.v v0, (a3)
                out("a0") _,
                in("a1") kernel_pointer,
                // Could be handled in rust but this is largely a sanity check
                inout("a2") loop_count => loop_count,
                in("a3") result.as_mut_ptr(),
            );
        }
        kernel_size -= 64;
        // println!("Loop performed: {}B, {}", kernel_size, loop_count - 1);
    }

    println!("Result: ");
    pretty_print_slice(unsafe { slice::from_raw_parts(result.as_ptr() as *mut u8, 32) });
    println!("Serial result: ");
    pretty_print_slice(&hash_kernel_serial());
    println!("Returned from vector hashing");
    panic!("HALT");
}

fn pretty_print_slice(bytes: &[u8]) {
    let mut counter = 0;
    let mut lines = 0;
    let column_limit = 16;
    print!("\r\n{:#04x} | ", lines * column_limit);
    for byte in bytes {
        if counter == column_limit {
            counter = 0;
            lines += 1;
            print!("\r\n{:#04x} | ", lines * column_limit);
        }
        counter += 1;
        print!("{:#04x} ", byte);
    }
    println!();
}
