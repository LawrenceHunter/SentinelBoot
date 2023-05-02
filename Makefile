##--------------------------------------------------------------------------------------------------
## Optional, user-provided configuration values
##--------------------------------------------------------------------------------------------------

BSP ?= visionfive
CLEAR ?= y
TOOLCHAIN ?= riscv64-unknown-elf-

# Default to a serial device name that is common in Linux.
DEV_SERIAL ?= /dev/ttyUSB0

##--------------------------------------------------------------------------------------------------
## BSP-specific configuration values
##--------------------------------------------------------------------------------------------------
QEMU_MISSING_STRING = "This board is not yet supported for QEMU."

ifeq ($(BSP),visionfive)
    TARGET            = riscv64gc-unknown-none-elf
    LOADER_BIN        = bootloader.img
    QEMU_BINARY       = qemu-system-riscv64
    QEMU_MACHINE_TYPE = sifive_u
    QEMU_RELEASE_ARGS = -cpu rv64 -smp 4 -m 128M
    OBJDUMP_BINARY    = $(TOOLCHAIN)objdump
    NM_BINARY         = $(TOOLCHAIN)nm
    READELF_BINARY    = $(TOOLCHAIN)readelf
    LD_SCRIPT_PATH    = $(shell pwd)/bsp/src/visionfive
	RUSTC_MISC_ARGS   = -C target-cpu=sifive-u74
endif

# Export for build.rs.
export LD_SCRIPT_PATH

##--------------------------------------------------------------------------------------------------
## Targets and Prerequisites
##--------------------------------------------------------------------------------------------------
LOADER_MANIFEST      = Cargo.toml
LOADER_LINKER_SCRIPT = bootloader.ld
LAST_BUILD_CONFIG    = target/$(BSP).build_config

LOADER_ELF      = target/$(TARGET)/release/bootloader
# This parses cargo's dep-info file.
# https://doc.rust-lang.org/cargo/guide/build-cache.html#dep-info-files
LOADER_ELF_DEPS = $(filter-out %: ,$(file < $(LOADER_ELF).d)) $(LOADER_MANIFEST) $(LAST_BUILD_CONFIG)

##--------------------------------------------------------------------------------------------------
## Command building blocks
##--------------------------------------------------------------------------------------------------
RUSTFLAGS = $(RUSTC_MISC_ARGS)                   \
	-C link-arg=--library-path=$(LD_SCRIPT_PATH) \
    -C link-arg=--script=$(LOADER_LINKER_SCRIPT)

RUSTFLAGS_PEDANTIC = $(RUSTFLAGS) \
    -D warnings                   \
    -D missing_docs

FEATURES      = --features $(BSP)
COMPILER_ARGS = --target=$(TARGET) \
    $(FEATURES)                    \
    --release

RUSTC_CMD   = cargo rustc $(COMPILER_ARGS)
DOC_CMD     = cargo doc $(COMPILER_ARGS)
CLIPPY_CMD  = cargo clippy $(COMPILER_ARGS)
OBJCOPY_CMD = rust-objcopy -O binary

EXEC_QEMU = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)

##--------------------------------------------------------------------------------------------------
## Targets
##--------------------------------------------------------------------------------------------------
.PHONY: all doc qemu qemu_halted clippy clean readelf objdump nm

all: $(LOADER_BIN)

##------------------------------------------------------------------------------
## Save the configuration as a file, so make understands if it changed.
##------------------------------------------------------------------------------
$(LAST_BUILD_CONFIG):
	@rm -f target/*.build_config
	@mkdir -p target
	@touch $(LAST_BUILD_CONFIG)

##------------------------------------------------------------------------------
## Compile the bootloader ELF
##------------------------------------------------------------------------------
$(LOADER_ELF): $(LOADER_ELF_DEPS)
	$(call color_header, "Compiling bootloader ELF - $(BSP)")
ifeq ($(CLEAR),y)
	clear
endif
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(RUSTC_CMD)

##------------------------------------------------------------------------------
## Generate the stripped bootloader binary
##------------------------------------------------------------------------------
$(LOADER_BIN): $(LOADER_ELF)
	$(call color_header, "Generating stripped binary")
	@$(OBJCOPY_CMD) $(LOADER_ELF) $(LOADER_BIN)
	$(call color_progress_prefix, "Name")
	@echo $(LOADER_BIN)
	$(call color_progress_prefix, "Size")
	$(call disk_usage_KiB, $(LOADER_BIN))

##------------------------------------------------------------------------------
## Generate the documentation
##------------------------------------------------------------------------------
doc:
	$(call color_header, "Generating docs")
	@$(DOC_CMD) --document-private-items --open

##------------------------------------------------------------------------------
## Run the bootloader in QEMU
##------------------------------------------------------------------------------
ifeq ($(QEMU_MACHINE_TYPE),) # QEMU is not supported for the board.

qemu:
	$(call color_header, "$(QEMU_MISSING_STRING)")

else # QEMU is supported.

qemu: $(LOADER_BIN)
	$(call color_header, "Launching QEMU")
	$(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -semihosting --semihosting-config \
	enable=on,target=native -nographic -serial mon:stdio -bios none \
	-kernel $(LOADER_BIN) -s
endif

ifeq ($(QEMU_MACHINE_TYPE),) # QEMU is not supported for the board.

qemu_halted:
	$(call color_header, "$(QEMU_MISSING_STRING)")

else # QEMU is supported.

qemu_halted: $(LOADER_BIN)
	$(call color_header, "Launching QEMU")
	$(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -semihosting --semihosting-config \
	enable=on,target=native -nographic -serial mon:stdio -bios none \
	-kernel $(LOADER_BIN) -s -S
endif

##------------------------------------------------------------------------------
## Run clippy
##------------------------------------------------------------------------------
clippy:
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)

##------------------------------------------------------------------------------
## Clean
##------------------------------------------------------------------------------
clean:
	rm -rf target $(LOADER_BIN)

##------------------------------------------------------------------------------
## Run readelf
##------------------------------------------------------------------------------
readelf: $(LOADER_ELF)
	$(call color_header, "Launching readelf")
	$(READELF_BINARY) --headers $(LOADER_ELF)

##------------------------------------------------------------------------------
## Run objdump
##------------------------------------------------------------------------------
objdump: $(LOADER_ELF)
	$(call color_header, "Launching objdump")
	@$(DOCKER_TOOLS) $(OBJDUMP_BINARY) --disassemble --demangle \
                --section .text     \
                --section .rodata   \
                $(LOADER_ELF) | rustfilt

##------------------------------------------------------------------------------
## Run nm
##------------------------------------------------------------------------------
nm: $(LOADER_ELF)
	$(call color_header, "Launching nm")
	$(NM_BINARY) --demangle --print-size $(LOADER_ELF) | sort | rustfilt
