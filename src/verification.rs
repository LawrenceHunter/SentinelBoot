use console::{print, println};
use core::slice;
use pelite::pe64::{self, Pe};
// #[cfg(not(feature = "qemu"))]
use sha2::{Digest, Sha256};

/* -------------------------------------------------------------------------- */
/*                            Serial Implementation                           */
/* -------------------------------------------------------------------------- */

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


/* -------------------------------------------------------------------------- */
/*                     Vector Cryptography Implementation                     */
/* -------------------------------------------------------------------------- */

// Temporary
#[cfg(feature = "qemu")]
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
const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, // 0-3
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, // 4-7
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, // 8-11
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, // 12-15
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, // 16-19
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, // 20-23
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, // 24-27
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, // 28-31
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, // 32-35
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, // 36-39
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, // 40-43
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, // 44-47
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, // 48-51
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, // 52-55
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, // 56-59
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2, // 60-63
];

#[cfg(feature = "qemu")]
fn hash_kernel() -> [u8; 32] {
    use core::arch::asm;

    let mut kernel_size = get_kernel_size();
    println!(
        "Kernel range: 0x{:X?} -> 0x{:X?}",
        bsp::memory::map::kernel::KERNEL,
        bsp::memory::map::kernel::KERNEL + kernel_size
    );
    let mut result: [u32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
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

    while kernel_size >= 32 {
        unsafe {
            asm!(
                /* ------------------------- Sanity ------------------------- */
                "addi a3, a3, 1",

                /* ------------------------- Setup -------------------------- */
                // Set vector configuration
                // vsetivli x0, 4, e32, m1, ta, ma

                // Load 512 bits of the message block into v10-v13 endian swaping
                // vle32.v v10, (a1)
                // vrev8.v v10, v10
                add a1, a1, 16,
                // vle32.v v11, (a1)
                // vrev8.v v11, v11
                add a1, a1, 16,
                // vle32.v v12, (a1)
                // vrev8.v v12, v12
                add a1, a1, 16,
                // vle32.v v13, (a1)
                // vrev8.v v13, v13

                // Load H[0..8]
                // v26 = v16 = {a[t],b[t],e[t],f[t]}
                // v27 = v17 = {c[t],d[t],g[t],h[t]}
                // vle32.v v16, (a0)
                addi a0, a0, 16,
                // vle32.v v17, (a0)

                // Capture initial H to allow computing H'
                // vmv.v.v v26, v16
                // vmv.v.v v27, v17

                // Set v0 for vmerge that replaces first word
                // v0.mask[i] = (i == 0 ? 1 : 0)
                // vid.v v0
                // vmseq.vi v0, v0, 0x0

                /* ----------------------- Quad rounds ---------------------- */
                // Round 0
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v10
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14
                // vmerge.vvm v14, v12, v11, v0
                // vsha2ms.vv v10, v14, v13

                // Round 1
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v11
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14
                // vmerge.vvm v14, v13, v12, v0
                // vsha2ms.vv v11, v14, v10

                // Round 2
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v12
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14
                // vmerge.vvm v14, v10, v13, v0
                // vsha2ms.vv v12, v14, v11

                // Round 3
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v13
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14
                // vmerge.vvm v14, v11, v10, v0
                // vsha2ms.vv v13, v14, v12

                // Round 4
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v10
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14
                // vmerge.vvm v14, v12, v11, v0
                // vsha2ms.vv v10, v14, v13

                // Round 5
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v11
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14
                // vmerge.vvm v14, v13, v12, v0
                // vsha2ms.vv v11, v14, v10

                // Round 6
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v12
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14
                // vmerge.vvm v14, v10, v13, v0
                // vsha2ms.vv v12, v14, v11

                // Round 7
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v13
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14
                // vmerge.vvm v14, v11, v10, v0
                // vsha2ms.vv v13, v14, v12

                // Round 8
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v10
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14
                // vmerge.vvm v14, v12, v11, v0
                // vsha2ms.vv v10, v14, v13

                // Round 9
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v11
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14
                // vmerge.vvm v14, v13, v12, v0
                // vsha2ms.vv v11, v14, v10

                // Round 10
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v12
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14
                // vmerge.vvm v14, v10, v13, v0
                // vsha2ms.vv v12, v14, v11

                // Round 11
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v13
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14
                // vmerge.vvm v14, v11, v10, v0
                // vsha2ms.vv v13, v14, v12

                // Round 12
                // We no longer generate new message schedules
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v10
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14

                // Round 13
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v11
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14

                // Round 14
                // vle32.v v15, (a2)
                addi a2, a2, 16,
                // vadd.vv v14, v15, v12
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14

                // Round 15
                // a2 increment not needed
                // vle32.v v15, (a2)
                // vadd.vv v14, v15, v13
                // vsha2cl.vv v17, v16, v14
                // vsha2ch.vv v16, v17, v14

                /* --------------------- Update the hash -------------------- */
                // vadd.vv v16, v26, v16
                // vadd.vv v17, v27, v17

                /* ---------------------- Save the hash --------------------- */
                // vse32.v v17, (a0)
                addi a0, a0, -16,
                // vse32.v v16, (a0)
                in("a0") bsp::memory::map::kernel::KERNEL + ((loops + 1) * 64),
                in("a1") result.as_mut_ptr(),
                in("a2") SHA256_ROUND_CONSTANTS.as_ptr(),
                inout("a3") loops => loops,
            );
        }
        kernel_size -= 64;
    }

    println!("Result: ({} iterations)", loops);

    // Temporary for comparison to known
    pretty_print_slice(
        unsafe {
            slice::from_raw_parts(result.as_ptr() as *mut u8, 32)
        }
    );
    println!("Serial result: ");
    pretty_print_slice(&hash_kernel_serial());
    println!("Returned from vector hashing");
    panic!("HALT");
}

/* -------------------------------------------------------------------------- */
/*                                Unified code                                */
/* -------------------------------------------------------------------------- */
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
