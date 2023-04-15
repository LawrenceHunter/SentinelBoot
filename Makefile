##--------------------------------------------------------------------------------------------------
## Optional, user-provided configuration values
##--------------------------------------------------------------------------------------------------

BSP ?= vsv
CLEAR ?= y

# Default to a serial device name that is common in Linux.
DEV_SERIAL ?= /dev/ttyUSB0

##--------------------------------------------------------------------------------------------------
## BSP-specific configuration values
##--------------------------------------------------------------------------------------------------
QEMU_MISSING_STRING = "This board is not yet supported for QEMU."

ifeq ($(BSP),vsv)
    TARGET            = riscv64gc-unknown-none-elf
    KERNEL_BIN        = kernel.img
    QEMU_BINARY       = qemu-system-riscv64
    QEMU_MACHINE_TYPE = sifive_u
    QEMU_RELEASE_ARGS = -serial stdio -display none
    OBJDUMP_BINARY    = riscv64-unknown-elf-objdump
    NM_BINARY         = riscv64-unknown-elf-nm
    READELF_BINARY    = riscv64-unknown-elf-readelf
    LD_SCRIPT_PATH    = $(shell pwd)/src/bsp/visionfive
	RUSTC_MISC_ARGS   = -C target-cpu=sifive-u74
endif

# Export for build.rs.
export LD_SCRIPT_PATH

##--------------------------------------------------------------------------------------------------
## Targets and Prerequisites
##--------------------------------------------------------------------------------------------------
KERNEL_MANIFEST      = Cargo.toml
KERNEL_LINKER_SCRIPT = kernel.ld
LAST_BUILD_CONFIG    = target/$(BSP).build_config

KERNEL_ELF      = target/$(TARGET)/release/kernel
# This parses cargo's dep-info file.
# https://doc.rust-lang.org/cargo/guide/build-cache.html#dep-info-files
KERNEL_ELF_DEPS = $(filter-out %: ,$(file < $(KERNEL_ELF).d)) $(KERNEL_MANIFEST) $(LAST_BUILD_CONFIG)

##--------------------------------------------------------------------------------------------------
## Command building blocks
##--------------------------------------------------------------------------------------------------
RUSTFLAGS = $(RUSTC_MISC_ARGS)                   \
	-C link-arg=--library-path=$(LD_SCRIPT_PATH) \
    -C link-arg=--script=$(KERNEL_LINKER_SCRIPT)

RUSTFLAGS_PEDANTIC = $(RUSTFLAGS) \
    -D warnings                   \
    -D missing_docs

FEATURES      = --features bsp_$(BSP)
COMPILER_ARGS = --target=$(TARGET) \
    $(FEATURES)                    \
    --release

RUSTC_CMD   = cargo rustc $(COMPILER_ARGS)
DOC_CMD     = cargo doc $(COMPILER_ARGS)
CLIPPY_CMD  = cargo clippy $(COMPILER_ARGS)
OBJCOPY_CMD = rust-objcopy \
    --strip-all            \
    -O binary

EXEC_QEMU = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)

##--------------------------------------------------------------------------------------------------
## Targets
##--------------------------------------------------------------------------------------------------
.PHONY: all doc qemu clippy clean readelf objdump nm

all: $(KERNEL_BIN)

##------------------------------------------------------------------------------
## Save the configuration as a file, so make understands if it changed.
##------------------------------------------------------------------------------
$(LAST_BUILD_CONFIG):
	@rm -f target/*.build_config
	@mkdir -p target
	@touch $(LAST_BUILD_CONFIG)

##------------------------------------------------------------------------------
## Compile the kernel ELF
##------------------------------------------------------------------------------
$(KERNEL_ELF): $(KERNEL_ELF_DEPS)
	$(call color_header, "Compiling kernel ELF - $(BSP)")
ifeq ($(CLEAR),y)
	clear
endif
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(RUSTC_CMD)

##------------------------------------------------------------------------------
## Generate the stripped kernel binary
##------------------------------------------------------------------------------
$(KERNEL_BIN): $(KERNEL_ELF)
	$(call color_header, "Generating stripped binary")
	@$(OBJCOPY_CMD) $(KERNEL_ELF) $(KERNEL_BIN)
	$(call color_progress_prefix, "Name")
	@echo $(KERNEL_BIN)
	$(call color_progress_prefix, "Size")
	$(call disk_usage_KiB, $(KERNEL_BIN))

##------------------------------------------------------------------------------
## Generate the documentation
##------------------------------------------------------------------------------
doc:
	$(call color_header, "Generating docs")
	@$(DOC_CMD) --document-private-items --open

##------------------------------------------------------------------------------
## Run the kernel in QEMU
##------------------------------------------------------------------------------
ifeq ($(QEMU_MACHINE_TYPE),) # QEMU is not supported for the board.

qemu:
	$(call color_header, "$(QEMU_MISSING_STRING)")

else # QEMU is supported.

qemu: $(KERNEL_BIN)
	$(call color_header, "Launching QEMU")
	$(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
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
	rm -rf target $(KERNEL_BIN)

##------------------------------------------------------------------------------
## Run readelf
##------------------------------------------------------------------------------
readelf: $(KERNEL_ELF)
	$(call color_header, "Launching readelf")
	@$(DOCKER_TOOLS) $(READELF_BINARY) --headers $(KERNEL_ELF)

##------------------------------------------------------------------------------
## Run objdump
##------------------------------------------------------------------------------
objdump: $(KERNEL_ELF)
	$(call color_header, "Launching objdump")
	@$(DOCKER_TOOLS) $(OBJDUMP_BINARY) --disassemble --demangle \
                --section .text     \
                --section .rodata   \
                $(KERNEL_ELF) | rustfilt

##------------------------------------------------------------------------------
## Run nm
##------------------------------------------------------------------------------
nm: $(KERNEL_ELF)
	$(call color_header, "Launching nm")
	@$(DOCKER_TOOLS) $(NM_BINARY) --demangle --print-size $(KERNEL_ELF) | sort | rustfilt
