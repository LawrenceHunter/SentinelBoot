##-----------------------------------------------------------------------------
## Optional, user-provided configuration values
##-----------------------------------------------------------------------------

BSP ?= visionfive
TOOLCHAIN ?= riscv64-unknown-linux-gnu-
DOCKER ?= y

##-----------------------------------------------------------------------------
## BSP-specific configuration values
##-----------------------------------------------------------------------------

ifeq ($(BSP),visionfive)
    LOADER_BIN        = bootloader.img
    QEMU_BINARY       = qemu-system-riscv64
    QEMU_MACHINE_TYPE = virt
    QEMU_RELEASE_ARGS = -cpu rv64 -smp 4 -m 4G
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
QEMU_ARGS   = $(QEMU_RELEASE_ARGS) -nographic -display none -serial stdio \
				 -s -drive format=raw,file=images/sdcard.img
##-----------------------------------------------------------------------------
## Targets
##-----------------------------------------------------------------------------
.PHONY: all doc qemu qemu_halted clippy clean readelf objdump nm test \
	call_stack geiger tftp image

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
	$(call color_header, "Generating docs")
ifeq ($(DOCKER),y)
	$(DOCKER_CMD) $(DOC_CMD)
else
	$(DOC_CMD)
endif

##------------------------------------------------------------------------------
## Run the bootloader in QEMU
##------------------------------------------------------------------------------
qemu: image
	$(call color_header, "Launching QEMU")
ifeq ($(DOCKER),y)
	$(DOCKER_CMD) $(EXEC_QEMU) $(QEMU_ARGS)
else
	$(EXEC_QEMU) $(QEMU_ARGS)
endif

qemu_halted: image
	$(call color_header, "Launching QEMU")
	$(EXEC_QEMU) $(QEMU_ARGS) -S
##------------------------------------------------------------------------------
## Run clippy
##------------------------------------------------------------------------------
clippy:
	$(call color_header, "Running cargo clippy")
ifeq ($(DOCKER),y)
	$(DOCKER_CMD) $(CLIPPY_CMD)
else
	$(CLIPPY_CMD)
endif

##------------------------------------------------------------------------------
## Clean
##------------------------------------------------------------------------------
clean:
	$(call color_header, "Cleaning")
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
	$(OBJDUMP_BINARY) --disassemble --demangle \
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

##------------------------------------------------------------------------------
## Generate call stack graph
##------------------------------------------------------------------------------
call_stack:
	$(call color_header, "Generating cargo call stack")
ifeq ($(DOCKER),y)
	$(DOCKER_CMD) cargo +nightly call-stack --bin bootloader --features \
	visionfive --target riscv64gc-unknown-none-elf > cg.dot ; \
	dot -Tsvg cg.dot > cg.svg && rm cg.dot
else
	cargo +nightly call-stack --bin bootloader --features visionfive --target \
	riscv64gc-unknown-none-elf > cg.dot ; dot -Tsvg cg.dot > cg.svg && rm cg.dot
endif

##------------------------------------------------------------------------------
## Execute cargo geiger
##------------------------------------------------------------------------------
geiger:
	$(call color_header, "Running cargo geiger")
ifeq ($(DOCKER),y)
	$(DOCKER_CMD) cargo geiger --target riscv64gc-unknown-none-elf --features \
	visionfive
else
	cargo geiger --target riscv64gc-unknown-none-elf --features visionfive
endif

##------------------------------------------------------------------------------
## Generate SD image
##------------------------------------------------------------------------------

image: $(LOADER_BIN)
ifeq ($(BSP),visionfive)
	cp ./bsp/src/visionfive/genimage.cfg .
endif
	$(call color_header, "Generating SD card image")
ifeq ($(DOCKER),y)
	$(DOCKER_CMD) genimage --inputpath $(shell pwd)
else
	genimage --inputpath $(shell pwd)
endif
	rm genimage.cfg
