set confirm off
set architecture riscv:rv64
set disassemble-next-line auto
set riscv use-compressed-breakpoints yes

# Image load start
b *0x80000000

# After initialising BSS
b *0x80000034
