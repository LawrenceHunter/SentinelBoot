##-----------------------------------------------------------------------------
## Optional, user-provided configuration values
##-----------------------------------------------------------------------------

BSP ?= qemu
TOOLCHAIN ?= riscv64-unknown-elf-
DOCKER ?= y
DEBUG ?= n

##-----------------------------------------------------------------------------
## BSP-specific configuration values
##-----------------------------------------------------------------------------

ifeq ($(BSP),qemu)
    LOADER_BIN        = bootloader.img
    QEMU_BINARY       = qemu-system-riscv64
    QEMU_MACHINE_TYPE = virt
    QEMU_RELEASE_ARGS = -cpu rv64 -smp 4 -m 256M
    OBJDUMP_BINARY    = $(TOOLCHAIN)objdump
    NM_BINARY         = $(TOOLCHAIN)nm
    READELF_BINARY    = $(TOOLCHAIN)readelf
	LD_PATH			  = riscv64/src/cpu/bootloader-raw.ld
endif

ifeq ($(BSP),qemu_tftp)
    LOADER_BIN        = bootloader.img
    QEMU_BINARY       = qemu-system-riscv64
    QEMU_MACHINE_TYPE = virt
    QEMU_RELEASE_ARGS = -cpu rv64 -smp 4 -m 256M
    OBJDUMP_BINARY    = $(TOOLCHAIN)objdump
    NM_BINARY         = $(TOOLCHAIN)nm
    READELF_BINARY    = $(TOOLCHAIN)readelf
	LD_PATH			  = riscv64/src/cpu/bootloader-raw.ld
endif

ifeq ($(BSP),visionfive)
    LOADER_BIN        = bootloader.img
    QEMU_BINARY       = qemu-system-riscv64
    QEMU_MACHINE_TYPE = virt
    QEMU_RELEASE_ARGS = -cpu rv64 -smp 4 -m 128M
    OBJDUMP_BINARY    = $(TOOLCHAIN)objdump
    NM_BINARY         = $(TOOLCHAIN)nm
    READELF_BINARY    = $(TOOLCHAIN)readelf
	LD_PATH           = riscv64/src/cpu/bootloader-u-boot.ld
endif

ifeq ($(BSP),unmatched)
    LOADER_BIN        = bootloader.img
    QEMU_BINARY       = qemu-system-riscv64
    QEMU_MACHINE_TYPE = sifive_u
    QEMU_RELEASE_ARGS = -cpu rv64 -smp 4 -m 128M
    OBJDUMP_BINARY    = $(TOOLCHAIN)objdump
    NM_BINARY         = $(TOOLCHAIN)nm
    READELF_BINARY    = $(TOOLCHAIN)readelf
	LD_PATH			  = riscv64/src/cpu/bootloader-u-boot.ld
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
ifeq ($(DEBUG),y)
	FEATURES = --features $(BSP),debug
else
	FEATURES = --features $(BSP)
endif
COMPILER_ARGS = $(FEATURES) --release

RUSTC_CMD   = cargo rustc $(COMPILER_ARGS)
DOC_CMD     = cargo doc $(COMPILER_ARGS) --features $(BSP) \
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
.PHONY: all doc qemu qemu_halted clippy clean readelf objdump nm test \
	call_stack geiger hyperfine

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
	cp $(LD_PATH) ./bootloader.ld
ifeq ($(DOCKER),y)
	$(call color_header, "Compiling bootloader ELF - $(BSP)")
	$(DOCKER_CMD) python3 gen_helper.py
	$(DOCKER_CMD) $(RUSTC_CMD)
else
	$(call color_header, "Compiling bootloader ELF - $(BSP)")
	python3 gen_helper.py
	$(RUSTC_CMD)
endif
	rm ./bootloader.ld
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
test: $(LOADER_BIN)
	timeout 5m .github/workflows/qemu_test.sh

##------------------------------------------------------------------------------
## Generate call stack graph
##------------------------------------------------------------------------------
call_stack:
ifeq ($(DOCKER),y)
	$(DOCKER_CMD) cargo +nightly call-stack --bin bootloader --features \
	$(BSP) --target riscv64gc-unknown-none-elf > cg.dot ; \
	dot -Tsvg cg.dot > cg.svg && rm cg.dot
else
	cargo +nightly call-stack --bin bootloader --features $(BSP) --target \
	riscv64gc-unknown-none-elf > cg.dot ; dot -Tsvg cg.dot > cg.svg && rm cg.dot
endif

##------------------------------------------------------------------------------
## Execute cargo geiger
##------------------------------------------------------------------------------
geiger:
	echo "# Safety Report" > .github/workflows/geiger.md
ifeq ($(DOCKER),y)
	$(DOCKER_CMD) cargo geiger --target riscv64gc-unknown-none-elf --features \
	$(BSP) --output-format GitHubMarkdown --update-readme \
	--readme-path .github/workflows/geiger.md
else
	cargo geiger --target riscv64gc-unknown-none-elf --features $(BSP) \
	--output-format GitHubMarkdown --update-readme \
	--readme-path .github/workflows/geiger.md
endif
	cat .github/workflows/geiger.md

##------------------------------------------------------------------------------
## Execute hyperfine
##------------------------------------------------------------------------------
hyperfine:
ifeq ($(DOCKER),y)
	$(DOCKER_CMD) touch test.md
	$(DOCKER_CMD) hyperfine --warmup 1 --show-output --export-markdown test.md ./.github/workflows/qemu_test.sh
	$(DOCKER_CMD) cat test.md
else
	touch test.md
	hyperfine --warmup 1 --show-output --export-markdown test.md ./.github/workflows/qemu_test.sh
	cat test.md
endif

##------------------------------------------------------------------------------
## Execute cargo expand
##------------------------------------------------------------------------------
expand:
ifeq ($(DOCKER),y)
	$(DOCKER_CMD) cargo expand --target riscv64gc-unknown-none-elf --features $(BSP)
else
	cargo expand --target riscv64gc-unknown-none-elf --features $(BSP)
endif
