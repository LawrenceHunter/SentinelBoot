use console::{print, println};
use core::slice;
use pelite::pe64::{self, Pe};
use sha2::{Digest, Sha256};

pub fn verify_kernel() -> Result<(), ed25519_compact::Error> {
    let mut hasher = Sha256::new();

    println!("Hashing stored kernel...");
    hash_kernel(&mut hasher);
    let hash = hasher.finalize();
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

fn hash_kernel(hasher: &mut Sha256) {
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
