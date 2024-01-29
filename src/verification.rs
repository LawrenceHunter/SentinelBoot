use console::{print, println};
use core::slice;
use pelite::pe64::{self, Pe};
#[cfg(not(feature = "qemu"))]
use sha2::{Digest, Sha256};
use core::arch::global_asm;

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
    println!("Kernel range: 0x{:X?} -> 0x{:X?}", bsp::memory::map::kernel::KERNEL, bsp::memory::map::kernel::KERNEL + kernel_size);
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
global_asm!(include_str!("vector_hash.s"));

#[cfg(feature = "qemu")]
extern "C" {
    fn hash_kernel_asm();
}

#[cfg(feature = "qemu")]
fn hash_kernel() -> [u8; 32] {
    let kernel_size = get_kernel_size();
    println!("Kernel range: 0x{:X?} -> 0x{:X?}", bsp::memory::map::kernel::KERNEL, bsp::memory::map::kernel::KERNEL + kernel_size);
    unsafe { hash_kernel_asm() };
    todo!()
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
