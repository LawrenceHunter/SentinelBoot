use console::{print, println};
use core::slice;
use pelite::pe64::{self, Pe};
// #[cfg(feature = "qemu")]
// use core::arch::asm;
#[cfg(not(feature = "qemu"))]
use sha2::{Digest, Sha256};

#[cfg(not(feature = "qemu"))]
fn hash_kernel() -> [u8; 32] {
    let mut hasher = Sha256::new();
    let mut offset = 0;
    let kernel_size: usize = get_kernel_size();
    let mut buff_size = min(4096, kernel_size);
    loop {
        let data = unsafe {
            slice::from_raw_parts(
                (bsp::memory::map::kernel::KERNEL + offset) as *mut u8,
                buff_size,
            )
        };
        hasher.update(data);
        offset += buff_size;
        buff_size = min(4096, kernel_size - offset);
        if buff_size == 0 {
            break;
        }
    }
    hasher.finalize().into()
}

#[cfg(feature = "qemu")]
const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428A2F98, 0x71374491, 0xB5C0FBCF, 0xE9B5DBA5, // 0-3
    0x3956C25B, 0x59F111F1, 0x923F82A4, 0xAB1C5ED5, // 4-7
    0xD807AA98, 0x12835B01, 0x243185BE, 0x550C7DC3, // 8-11
    0x72BE5D74, 0x80DEB1FE, 0x9BDC06A7, 0xC19BF174, // 12-15
    0xE49B69C1, 0xEFBE4786, 0x0FC19DC6, 0x240CA1CC, // 16-19
    0x2DE92C6F, 0x4A7484AA, 0x5CB0A9DC, 0x76F988DA, // 20-23
    0x983E5152, 0xA831C66D, 0xB00327C8, 0xBF597FC7, // 24-27
    0xC6E00BF3, 0xD5A79147, 0x06CA6351, 0x14292967, // 28-31
    0x27B70A85, 0x2E1B2138, 0x4D2C6DFC, 0x53380D13, // 32-35
    0x650A7354, 0x766A0ABB, 0x81C2C92E, 0x92722C85, // 36-39
    0xA2BFE8A1, 0xA81A664B, 0xC24B8B70, 0xC76C51A3, // 40-43
    0xD192E819, 0xD6990624, 0xF40E3585, 0x106AA070, // 44-47
    0x19A4C116, 0x1E376C08, 0x2748774C, 0x34B0BCB5, // 48-51
    0x391C0CB3, 0x4ED8AA4A, 0x5B9CCA4F, 0x682E6FF3, // 52-55
    0x748F82EE, 0x78A5636F, 0x84C87814, 0x8CC70208, // 56-59
    0x90BEFFFA, 0xA4506CEB, 0xBEF9A3F7, 0xC67178F2, // 60-63
];

#[cfg(feature = "qemu")]
fn asm_hash(_a0: *mut usize, _a1: *mut usize, _a2: *mut usize) {
    todo!()
}

#[cfg(feature = "qemu")]
fn hash_kernel() -> [u8; 32] {
    todo!()
}

// --------------------------------------------------------------------------
// Unified code
// --------------------------------------------------------------------------

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

fn min(a: usize, b: usize) -> usize {
    if a < b {
        return a;
    }
    b
}
