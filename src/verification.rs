use console::{print, println};
#[cfg(feature = "qemu")]
use core::arch::asm;
use core::slice;
use pelite::pe64::{self, Pe};
// #[cfg(not(feature = "qemu"))]
use sha2::{Digest, Sha256};

// --------------------------------------------------------------------------
// Serial Implementation
// --------------------------------------------------------------------------

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

// --------------------------------------------------------------------------
// Vector Cryptography Implementation
// --------------------------------------------------------------------------

// Temporary
#[cfg(feature = "qemu")]
fn hash_kernel_serial() -> [u8; 32] {
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

fn asm_hash(a0: *mut usize, a1: *mut usize, a2: *mut usize) {
    unsafe {
        asm!(
            /* ------------------------- Sanity ------------------------- */
            "addi a3, a3, 1",

            /* ------------------------- Setup -------------------------- */
            // Set vector configuration
            ".word 0xcd027057", // vsetivli zero,4,e32,m1,ta,ma

            // Load 512 bits of the message block into v10-v13 endian swaping
            ".word 0x0205e507", // vle32.v v10,(a1)
            ".word 0x4aa4a557", // vrev8.v v10 v10
            "add a1, a1, 16",
            ".word 0x0205e587", // vle32.v v11,(a1)
            ".word 0x4ab4a5d7", // vrev8.v v11 v11
            "add a1, a1, 16",
            ".word 0x0205e607", // vle32.v v12,(a1)
            ".word 0x4ac4a657", // vrev8.v v12 v12
            "add a1, a1, 16",
            ".word 0x0205e687", // vle32.v v13,(a1)
            ".word 0x4ad4a6d7", // vrev8.v v13 v13

            /* ---------------------- Round loop ------------------------ */
            // Load H[0..8]
            // v26 = v16 = {a[t],b[t],e[t],f[t]}
            // v27 = v17 = {c[t],d[t],g[t],h[t]}
            ".word 0x02056807", // vle32.v v16,(a0)
            "addi a0, a0, 16",
            ".word 0x02056887", // vle32.v v17,(a0)

            // Capture initial H to allow computing H'
            ".word 0x5e080d57", // vmv.v.v v26,v16
            ".word 0x5e088dd7", // vmv.v.v v27,v17

            // Set v0 for vmerge that replaces first word
            // v0.mask[i] = (i == 0 ? 1 : 0)
            ".word 0x5208a057", // vid.v v0
            ".word 0x62003057", // vmseq.vi v0,v0,0

            /* ----------------------- Quad rounds ---------------------- */
            // Round 0
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f50757", // vadd.vv v14,v15,v10
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14
            ".word 0x5cc58757", // vmerge.vvm v14,v12,v11,v0
            ".word 0xb6e6a577", // vsha2ms.vv v10 v14 v13

            // Round 1
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f58757", // vadd.vv v14,v15,v11
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14
            ".word 0x5cd60757", // vmerge.vvm v14,v13,v12,v0
            ".word 0xb6e525f7", // vsha2ms.vv v11 v14 v10

            // Round 2
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f60757", // vadd.vv v14,v15,v12
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14
            ".word 0x5ca68757", // vmerge.vvm v14,v10,v13,v0
            ".word 0xb6e5a677", // vsha2ms.vv v12 v14 v11

            // Round 3
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f68757", // vadd.vv v14,v15,v13
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14
            ".word 0x5cb50757", // vmerge.vvm v14,v11,v10,v0
            ".word 0xb6e626f7", // vsha2ms.vv v13 v14 v12

            // Round 4
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f50757", // vadd.vv v14,v15,v10
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14
            ".word 0x5cc58757", // vmerge.vvm v14,v12,v11,v0
            ".word 0xb6e6a577", // vsha2ms.vv v10 v14 v13

            // Round 5
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f58757", // vadd.vv v14,v15,v11
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14
            ".word 0x5cd60757", // vmerge.vvm v14,v13,v12,v0
            ".word 0xb6e525f7", // vsha2ms.vv v11 v14 v10

            // Round 6
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f60757", // vadd.vv v14,v15,v12
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14
            ".word 0x5ca68757", // vmerge.vvm v14,v10,v13,v0
            ".word 0xb6e5a677", // vsha2ms.vv v12 v14 v11

            // Round 7
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f68757", // vadd.vv v14,v15,v13
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14
            ".word 0x5cb50757", // vmerge.vvm v14,v11,v10,v0
            ".word 0xb6e626f7", // vsha2ms.vv v13 v14 v12

            // Round 8
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f50757", // vadd.vv v14,v15,v10
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14
            ".word 0x5cc58757", // vmerge.vvm v14,v12,v11,v0
            ".word 0xb6e6a577", // vsha2ms.vv v10 v14 v13

            // Round 9
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f58757", // vadd.vv v14,v15,v11
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14
            ".word 0x5cd60757", // vmerge.vvm v14,v13,v12,v0
            ".word 0xb6e525f7", // vsha2ms.vv v11 v14 v10

            // Round 10
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f60757", // vadd.vv v14,v15,v12
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14
            ".word 0x5ca68757", // vmerge.vvm v14,v10,v13,v0
            ".word 0xb6e5a677", // vsha2ms.vv v12 v14 v11

            // Round 11
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f68757", // vadd.vv v14,v15,v13
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14
            ".word 0x5cb50757", // vmerge.vvm v14,v11,v10,v0
            ".word 0xb6e626f7", // vsha2ms.vv v13 v14 v12

            // Round 12
            // We no longer generate new message schedules
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f50757", // vadd.vv v14,v15,v10
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14

            // Round 13
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f58757", // vadd.vv v14,v15,v11
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14

            // Round 14
            ".word 0x02066787", // vle32.v v15,(a2)
            "addi a2, a2, 16",
            ".word 0x02f60757", // vadd.vv v14,v15,v12
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14

            // Round 15
            // a2 increment not needed
            ".word 0x02066787", // vle32.v v15,(a2)
            ".word 0x02f68757", // vadd.vv v14,v15,v13
            ".word 0xbf0728f7", // vsha2cl.vv v17 v16 v14
            ".word 0xbb172877", // vsha2ch.vv v16 v17 v14

            /* ----------------------- Update hash ---------------------- */
            ".word 0x03a80857", // vadd.vv v16, v26, v16
            ".word 0x03b888d7", // vadd.vv v17, v27, v17

            /* ------------------------ Save hash ----------------------- */
            ".word 0x020568a7", // vse32.v v17,(a0)
            "addi a0, a0, -16",
            ".word 0x02056827", // vse32.v v16,(a0)
            in("a0") a0,
            in("a1") a1,
            in("a2") a2,
        );
    }
}

#[cfg(feature = "qemu")]
fn hash_kernel() -> [u8; 32] {
    let kernel_size = get_kernel_size();

    println!(
        "Kernel range: 0x{:X?} -> 0x{:X?}",
        bsp::memory::map::kernel::KERNEL,
        bsp::memory::map::kernel::KERNEL + kernel_size
    );

    // Initialise with SHA256 initial Hash
    let mut result: [u32; 8] = [
        0x9B05688C, // F
        0x510E527F, // E
        0xBB67AE85, // B
        0x6A09E667, // A
        0x5BE0CD19, // H
        0x1F83D9AB, // G
        0xA54FF53A, // D
        0x3C6EF372, // C
    ];

    // Used for a sanity check
    let mut loops = 0;

    // ? Note this code cannot be used but is here for completeness
    //      The U-boot which sits before QEMU to prevent the need to implement
    //      a driver for tftp booting does not support vector operations and
    //      as it hands operation to use in supervisor mode it is not possible
    //      for us to enable this bit as such for development the require_rvv
    //      check in QEMU is hardcode to return true.
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
    // ? Rust's RISC-V targets do not support vector operations nor vector
    // ? cryptography operations as such the instructions are pre-assembled
    // ? into their binary equivalents and hardcoded the comments represent
    // ? the assembled instruction this solely bypasses the assembler and does
    // ? not further negate security guarantees.

    // Reference: RISC-V Cryptographic Extensions Vector Code Sample
    // The code sample is part of the riscv-crypto project on GitHub
    // commit 6589bcd6edb5abd91e758a67b28ae05b347c0470.
    // See: https://github.com/riscv/riscv-crypto/blob/
    //  6589bcd6edb5abd91e758a67b28ae05b347c0470/doc/vector/code-samples/zvknh.s

    let mut size_left = kernel_size;
    println!(
        "Kernel range: 0x{:X?} -> 0x{:X?}",
        bsp::memory::map::kernel::KERNEL,
        bsp::memory::map::kernel::KERNEL + kernel_size
    );
    while size_left >= 64 {
        println!(
            "\r Hashing kernel range: 0x{:X?} -> 0x{:X?}",
            (bsp::memory::map::kernel::KERNEL + (loops * 64)),
            (bsp::memory::map::kernel::KERNEL + (loops + 1 * 64))
        );
        asm_hash(
            result.as_mut_ptr() as *mut usize,
            (bsp::memory::map::kernel::KERNEL + (loops * 64)) as *mut usize,
            SHA256_ROUND_CONSTANTS.as_ptr() as *mut usize,
        );
        loops += 1;
        size_left -= 64;
    }

    println!("\nResult: ({} iterations)", loops);

    // Padding step
    let unaligned_size = kernel_size % 64;
    let mut final_bytes: [u8; 64] = [0; 64];
    for i in 0..unaligned_size {
        final_bytes[i] = unsafe {
            core::ptr::read(
                (bsp::memory::map::kernel::KERNEL
                    + (64 * (kernel_size / 64))
                    + i) as *mut u8,
            )
        };
    }
    final_bytes[unaligned_size] = 0x80;
    let final_size: u64 = (kernel_size as u64) * 8;
    let mut index = 56;
    for byte in final_size.to_be_bytes() {
        final_bytes[index] = byte;
        index += 1;
    }

    asm_hash(
        result.as_mut_ptr() as *mut usize,
        final_bytes.as_ptr() as *mut usize,
        SHA256_ROUND_CONSTANTS.as_ptr() as *mut usize,
    );
    loops += 1;

    result = result.map(|x: u32| x.swap_bytes());
    result = [
        result[3], result[2], result[7], result[6], result[1], result[0],
        result[5], result[4],
    ];

    println!("Returned from vector hashing");
    println!("\nResult: ({} iterations)", loops);

    // Temporary for comparison to known
    println!("Vector result");
    pretty_print_slice(unsafe {
        slice::from_raw_parts(result.as_ptr() as *mut u8, 32)
    });

    println!("\nSerial result: ");
    pretty_print_slice(&hash_kernel_serial());

    panic!("HALT");
}

// --------------------------------------------------------------------------
// Unified code
// --------------------------------------------------------------------------
fn pretty_print_slice(bytes: &[u8]) {
    let mut counter = 0;
    let mut lines = 0;
    let column_limit = 16;
    print!("\r{:#04x} | ", lines * column_limit);
    for byte in bytes {
        if counter == column_limit {
            counter = 0;
            lines += 1;
            print!("\r\n{:#04X} | ", lines * column_limit);
        }
        counter += 1;
        print!("{:#04X} ", byte);
    }
    println!();
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

fn min(a: usize, b: usize) -> usize {
    if a < b {
        return a;
    }
    b
}
