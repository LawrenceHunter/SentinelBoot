use console::{print, println};
use core::slice;
use pelite::pe64::{self, Pe};
#[cfg(not(feature = "qemu"))]
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
    kernel_size
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
    kernel_size = 256;
    println!("Attempting vector hashing - kernel size: {}B", kernel_size);
    while kernel_size >= 32 {
        let kernel_pointer = bsp::memory::map::kernel::KERNEL + (loop_count * 32);
        unsafe {
            asm!(
                "li a0, 4",
                "addi a2, a2, 1",
                "addi a3, a3, -32",
                ".word 0x18572D7", // vsetvli t0, a0, e64, m1, tu, mu
                ".word 0xE00033", // vle64.v v0, (a1)
                ".word 0xF08033", // mv v0, a4
                ".word 0x1010033", // mv v1, a5
                ".word 0x1118033", // mv v2, a6
                ".word 0x205F207", // mv v3, a7
                ".word 0x205F407", // vle64.v v8, (a1)
                ".word 0xB6822077", // vsha2ms.vv v0, v8, v4
                ".word 0x70033", // mv a4, v0
                ".word 0x178033", // mv a5, v1
                ".word 0x280033", // mv a6, v2
                ".word 0x388033", // mv a7, v3
                out("a0") _,
                inout("a1") kernel_pointer => _,
                inout("a2") loop_count => loop_count,
                inout("a3") kernel_size => kernel_size,
                inout("a4") result[0] => result[0],
                inout("a5") result[1] => result[1],
                inout("a6") result[2] => result[2],
                inout("a7") result[3] => result[3],
            );
        }
        println!("Loop performed: {}B, {}", kernel_size, loop_count);
    }
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
