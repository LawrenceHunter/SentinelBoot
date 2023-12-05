use sha2::{Digest, Sha256};
use core::slice;
use console::{println, print};

pub fn verify_kernel() -> Result<(), ed25519_compact::Error> {
    let mut hasher = Sha256::new();

    get_kernel_size();

    println!("Hashing stored kernel...");
    hash_kernel(&mut hasher);
    let hash = hasher.finalize();
    println!("Stored kernel hashed:");
    pretty_print_slice(hash.as_slice());

    println!("Loading server public key...");
    let public_key = ed25519_compact::PublicKey::from_slice(crate::helper::PUBLIC_KEY).unwrap();
    println!("Loaded server public key:");
    pretty_print_slice(public_key.as_slice());

    println!("Loading kernel signature...");
    let signature_bytes = unsafe { slice::from_raw_parts(
        (bsp::memory::map::kernel::SIGNATURE)
            as *mut u8,
        64,
    ) };
    let signature = ed25519_compact::Signature::from_slice(signature_bytes).unwrap();
    println!("Loaded kernel signature:");
    pretty_print_slice(signature.as_slice());

    println!("Verifying stored kernel...");
    public_key.verify(hash.as_slice(), &signature)
}

fn get_kernel_size() {
    let data = unsafe {
        slice::from_raw_parts(
            (bsp::memory::map::kernel::KERNEL)
                as *mut u8,
            4096,
        )
    };
    let minimal = elf::ElfBytes::<elf::endian::LittleEndian>::minimal_parse(data);
    println!("{:?}", minimal);
}

fn hash_kernel(hasher: &mut Sha256) {
    let mut offset = 0;
    let buff_size = 4096;
    loop {
        let data = unsafe {
            slice::from_raw_parts(
                (bsp::memory::map::kernel::KERNEL + (offset * buff_size))
                    as *mut u8,
                buff_size,
            )
        };
        // This is the problem how at runtime do I detect 13365248 bytes
        if (offset * buff_size) >= 13365248 {
            break;
        }
        hasher.update(data);
        offset += 1;
    }
}

fn pretty_print_slice(bytes: &[u8]) {
    let mut counter = 0;
    let mut lines = 0;
    let column_limit = 16;
    print!("{:#04x} | ", lines * column_limit);
    for byte in bytes {
        if counter == column_limit {
            counter = 0;
            lines += 1;
            print!("\n{:#04x} | ", lines * column_limit);
        }
        counter += 1;
        print!("{:#04x} ", byte);
    }
    println!();
}

pub fn dump_memory(start: usize, size: usize) {
    let data = unsafe {
        slice::from_raw_parts(
            (start)
                as *mut u8,
            size,
        )
    };
    pretty_print_slice(data);
}

pub fn dump_bsp_memory() {
    let dump_length = 64;
    println!("Dumping {} bytes from signature start...", dump_length);
    dump_memory(crate::bsp::memory::map::kernel::SIGNATURE, dump_length);
    println!("Dumping {} bytes from kernel start...", dump_length);
    dump_memory(crate::bsp::memory::map::kernel::KERNEL, dump_length);
    println!("Dumping {} bytes from dtb start...", dump_length);
    dump_memory(crate::bsp::memory::map::kernel::DTB, dump_length);
    println!("Dumping {} bytes from ramfs start...", dump_length);
    dump_memory(crate::bsp::memory::map::kernel::RAMFS, dump_length);
}
