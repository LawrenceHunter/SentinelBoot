##-----------------------------------------------------------------------------
## Optional, user-provided configuration values
##-----------------------------------------------------------------------------

BSP ?= visionfive
TOOLCHAIN ?= riscv64-unknown-elf-
DOCKER ?= y

##-----------------------------------------------------------------------------
## BSP-specific configuration values
##-----------------------------------------------------------------------------

ifeq ($(BSP),visionfive)
    LOADER_BIN        = bootloader.img
    QEMU_BINARY       = qemu-system-riscv64
    QEMU_MACHINE_TYPE = virt
    QEMU_RELEASE_ARGS = -cpu rv64 -smp 4 -m 128M
    OBJDUMP_BINARY    = $(TOOLCHAIN)objdump
    NM_BINARY         = $(TOOLCHAIN)nm
    READELF_BINARY    = $(TOOLCHAIN)readelf
endif

##-----------------------------------------------------------------------------
## Targets and Prerequisites
##-----------------------------------------------------------------------------
LOADER_MANIFEST      = Cargo.toml
LAST_BUILD_CONFIG    = target/$(BSP).build_config

LOADER_ELF      = target/riscv64gc-unknown-none-elf/release/bootloader
# This parses cargo's dep-info file.
# https://doc.rust-lang.org/cargo/guide/build-cache.html#dep-info-files
LOADER_ELF_DEPS = $(filter-out %: ,$(file < $(LOADER_ELF).d)) \
					$(LOADER_MANIFEST) $(LAST_BUILD_CONFIG)

##-----------------------------------------------------------------------------
## Command building blocks
##-----------------------------------------------------------------------------
FEATURES      = --features $(BSP)
COMPILER_ARGS = $(FEATURES) --release

RUSTC_CMD   = cargo rustc $(COMPILER_ARGS)
DOC_CMD     = cargo doc $(COMPILER_ARGS) --all-features \
				--document-private-items --workspace
CLIPPY_CMD  = cargo clippy $(COMPILER_ARGS) -- -A clippy::modulo_one
OBJCOPY_CMD = rust-objcopy -O binary
EXEC_QEMU   = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
DOCKER_CMD  = docker build --tag bootloader --file Dockerfile . && \
				docker run -v $(shell pwd):$(shell pwd) \
				-w $(shell pwd) bootloader:latest
QEMU_ARGS   = $(QEMU_RELEASE_ARGS) -nographic -display none -serial mon:stdio \
				-bios none -kernel $(LOADER_BIN) -s
##-----------------------------------------------------------------------------
## Targets
##-----------------------------------------------------------------------------
.PHONY: all doc qemu qemu_halted clippy clean readelf objdump nm test

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
ifeq ($(DOCKER),y)
	$(call color_header, "Compiling bootloader ELF - $(BSP)")
	$(DOCKER_CMD) $(RUSTC_CMD)
else
	$(call color_header, "Compiling bootloader ELF - $(BSP)")
	$(RUSTC_CMD)
endif
##------------------------------------------------------------------------------
## Generate the stripped bootloader binary
##------------------------------------------------------------------------------
$(LOADER_BIN): $(LOADER_ELF)
ifeq ($(DOCKER),y)
	$(call color_header, "Generating stripped binary")
	$(DOCKER_CMD) $(OBJCOPY_CMD) $(LOADER_ELF) $(LOADER_BIN)
	$(call color_progress_prefix, "Name")
	$(DOCKER_CMD) echo $(LOADER_BIN)
	$(call color_progress_prefix, "Size")
	$(call disk_usage_KiB, $(LOADER_BIN))
else
	$(call color_header, "Generating stripped binary")
	@$(OBJCOPY_CMD) $(LOADER_ELF) $(LOADER_BIN)
	$(call color_progress_prefix, "Name")
	@echo $(LOADER_BIN)
	$(call color_progress_prefix, "Size")
	$(call disk_usage_KiB, $(LOADER_BIN))
endif

##------------------------------------------------------------------------------
## Generate the documentation
##------------------------------------------------------------------------------
doc:
ifeq ($(DOCKER),y)
	$(call color_header, "Generating docs")
	$(DOCKER_CMD) $(DOC_CMD)
else
	$(call color_header, "Generating docs")
	$(DOC_CMD)
endif

##------------------------------------------------------------------------------
## Run the bootloader in QEMU
##------------------------------------------------------------------------------
qemu: $(LOADER_BIN)
ifeq ($(DOCKER),y)
	$(call color_header, "Launching QEMU")
	$(DOCKER_CMD) $(EXEC_QEMU) $(QEMU_ARGS)
else
	$(call color_header, "Launching QEMU")
	$(EXEC_QEMU) $(QEMU_ARGS)
endif

qemu_halted: $(LOADER_BIN)
	$(call color_header, "Launching QEMU")
	$(EXEC_QEMU) $(QEMU_ARGS) -S
##------------------------------------------------------------------------------
## Run clippy
##------------------------------------------------------------------------------
clippy:
ifeq ($(DOCKER),y)
	$(DOCKER_CMD) $(CLIPPY_CMD)
else
	$(CLIPPY_CMD)
endif

##------------------------------------------------------------------------------
## Clean
##------------------------------------------------------------------------------
clean:
ifeq ($(DOCKER),y)
	$(DOCKER_CMD) rm -rf target $(LOADER_BIN)
else
	rm -rf target $(LOADER_BIN)
endif

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

##------------------------------------------------------------------------------
## Run tests
##------------------------------------------------------------------------------
test:
	timeout 5m .github/workflows/qemu_test.sh
