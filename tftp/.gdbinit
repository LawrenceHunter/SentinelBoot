define mystep
    x/4i $pc-4
    stepi
    x/4i $pc-4
end

# /home/l/Thesis/bootloader/tftp/buildroot

define main
    set arch riscv:rv64
    set mem inaccessible-by-default off
    add-symbol-file /home/l/Thesis/bootloader/tftp/buildroot/output/build/linux-5.10.162-cip24/vmlinux
    target remote :1234

    # start_kernel
    b *0x80202000
    b *0x80200000

    # b *0x8020204a
    # b *0x8075544c
    # b *0x802050de
    # b *0x80201000
    # b *0x80201038

    # efi_header_end() - confirmed
    b *0x80202076
    # we make it back from this - sometimes?

    # secondary_start_common()
    b *0x802020d0

    # efi_header_end() in secondary_start_common
    b *0x802010a8
    # This jumps to 0x80201000
    # 0x80201038: sfence.vma
    # After this instruction "cannot access memory"
    # Seem to lose all control with gdb here
    # Using watch $pc
    # 0x8020105c -> 0x802010cc lose control
    # 0x8020105c is a ret
    # 0x802010cc is a wfi 
    b *0x80201038
    b *0x8020105c

    # secondary_start_common setup_trap_vector (after efi_header_end)
    b *0x802010ac

    # setup_trap_vector()
    b *0x8020207a
    # soc_early_init(...);
    b *0x80202092
    # start_kernel(...);
    b *0x80202096
    # start_kernel(...) entry
    b *0x80202764

    # setup_trap_vector
    b *0x802010b8

    c
end

main

