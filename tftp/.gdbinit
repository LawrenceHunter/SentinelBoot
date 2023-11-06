define mystep
    x/4i $pc-4
    stepi
    x/4i $pc-4
end

define main
    set arch riscv:rv64
    target remote :1234

    b *0x80202000
    c
end

main
