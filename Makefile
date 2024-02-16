# ---------------------------------------------------------------------------- #
#                 Optional, user-provided configuration values                 #
# ---------------------------------------------------------------------------- #
BSP ?= qemu
TOOLCHAIN ?= riscv64-unknown-elf-
DOCKER ?= y
DEBUG ?= n
CLEAR ?= y

# ---------------------------------------------------------------------------- #
#                       BSP-specific configuration values                      #
# ---------------------------------------------------------------------------- #
ifeq ($(BSP),qemu_vector)
    LOADER_BIN        = bootloader
    QEMU_BINARY       = qemu-system-riscv64
    QEMU_MACHINE_TYPE = virt
    QEMU_RELEASE_ARGS = -smp 4 -m 256M
    OBJDUMP_BINARY    = $(TOOLCHAIN)objdump
    NM_BINARY         = $(TOOLCHAIN)nm
    READELF_BINARY    = $(TOOLCHAIN)readelf
	LD_PATH			  = riscv64/src/cpu/bootloader-qemu.ld
endif

ifeq ($(BSP),qemu)
    LOADER_BIN        = bootloader
    QEMU_BINARY       = qemu-system-riscv64
    QEMU_MACHINE_TYPE = virt
    QEMU_RELEASE_ARGS = -smp 4 -m 256M
    OBJDUMP_BINARY    = $(TOOLCHAIN)objdump
    NM_BINARY         = $(TOOLCHAIN)nm
    READELF_BINARY    = $(TOOLCHAIN)readelf
	LD_PATH			  = riscv64/src/cpu/bootloader-qemu.ld
endif

ifeq ($(BSP),qemu_alloc)
    LOADER_BIN        = bootloader
    QEMU_BINARY       = qemu-system-riscv64
    QEMU_MACHINE_TYPE = virt
    QEMU_RELEASE_ARGS = -smp 4 -m 256M
    OBJDUMP_BINARY    = $(TOOLCHAIN)objdump
    NM_BINARY         = $(TOOLCHAIN)nm
    READELF_BINARY    = $(TOOLCHAIN)readelf
	LD_PATH			  = riscv64/src/cpu/bootloader-qemu.ld
endif

ifeq ($(BSP),visionfive)
    LOADER_BIN        = bootloader
    QEMU_BINARY       = qemu-system-riscv64
    QEMU_MACHINE_TYPE = virt
    QEMU_RELEASE_ARGS = -smp 4 -m 128M
    OBJDUMP_BINARY    = $(TOOLCHAIN)objdump
    NM_BINARY         = $(TOOLCHAIN)nm
    READELF_BINARY    = $(TOOLCHAIN)readelf
	LD_PATH           = riscv64/src/cpu/bootloader-u-boot.ld
endif

ifeq ($(BSP),unmatched)
    LOADER_BIN        = bootloader
    QEMU_BINARY       = qemu-system-riscv64
    QEMU_MACHINE_TYPE = sifive_u
    QEMU_RELEASE_ARGS = -smp 4 -m 128M
    OBJDUMP_BINARY    = $(TOOLCHAIN)objdump
    NM_BINARY         = $(TOOLCHAIN)nm
    READELF_BINARY    = $(TOOLCHAIN)readelf
	LD_PATH			  = riscv64/src/cpu/bootloader-u-boot.ld
endif

# ---------------------------------------------------------------------------- #
#                           Targets and Prerequisites                          #
# ---------------------------------------------------------------------------- #
LOADER_MANIFEST      = Cargo.toml
LAST_BUILD_CONFIG    = target/$(BSP).build_config

LOADER_ELF      = target/riscv64gc-unknown-none-elf/release/bootloader
# This parses cargo's dep-info file.
# https://doc.rust-lang.org/cargo/guide/build-cache.html#dep-info-files
LOADER_ELF_DEPS = $(filter-out %: ,$(file < $(LOADER_ELF).d)) \
					$(LOADER_MANIFEST) $(LAST_BUILD_CONFIG)

# ---------------------------------------------------------------------------- #
#                            Command building blocks                           #
# ---------------------------------------------------------------------------- #
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

DOCKER_MINIMAL_CMD = docker build --tag rust-minimal --file Dockerfile \
						--target rust-minimal . && \
					docker run -v $(shell pwd):$(shell pwd) \
						-w $(shell pwd) rust-minimal:latest
DOCKER_FULL_CMD    = docker build --tag rust-full --file Dockerfile \
							--target rust-full . && \
					docker run -v $(shell pwd):$(shell pwd) \
						-w $(shell pwd) rust-full:latest

QEMU_CPU_FLAGS  = -cpu rv64,v=true,vlen=1024,rvv_ma_all_1s=true,\
						rvv_ta_all_1s=true,x-zvbb=true,x-zvbc=true,x-zvknhb=true
QEMU_ARGS   	= $(QEMU_CPU_FLAGS) $(QEMU_RELEASE_ARGS) -nographic \
					-display none -serial mon:stdio \
					-bios bootloader -s \
					-monitor unix:qemu-monitor-socket,server,nowait

# ---------------------------------------------------------------------------- #
#                                    Targets                                   #
# ---------------------------------------------------------------------------- #
.PHONY: all doc qemu qemu_halted clippy clean readelf objdump nm test \
	call_stack geiger hyperfine

all: $(LOADER_BIN)

# ---------------------------------------------------------------------------- #
#      Save the configuration as a file, so make understands if it changed     #
# ---------------------------------------------------------------------------- #
$(LAST_BUILD_CONFIG):
	@rm -f target/*.build_config
	@mkdir -p target
	@touch $(LAST_BUILD_CONFIG)

# ---------------------------------------------------------------------------- #
#                          Compile the bootloader ELF                          #
# ---------------------------------------------------------------------------- #
$(LOADER_ELF): $(LOADER_ELF_DEPS)
ifeq ($(CLEAR),y)
	clear && tmux clear-history || true
endif
	cp $(LD_PATH) ./bootloader.ld
ifeq ($(DOCKER),y)
	$(call color_header, "Compiling bootloader ELF - $(BSP)")
	$(DOCKER_MINIMAL_CMD) cargo vendor
	$(DOCKER_MINIMAL_CMD) python3 gen_helper.py
	$(DOCKER_MINIMAL_CMD) $(RUSTC_CMD)
else
	$(call color_header, "Compiling bootloader ELF - $(BSP)")
	cargo vendor
	python3 gen_helper.py
	$(RUSTC_CMD)
endif
	rm ./bootloader.ld

# ---------------------------------------------------------------------------- #
#                    Generate the stripped bootloader binary                   #
# ---------------------------------------------------------------------------- #
$(LOADER_BIN): $(LOADER_ELF)
ifeq ($(DOCKER),y)
	$(call color_header, "Generating stripped binary")
	$(DOCKER_MINIMAL_CMD) $(OBJCOPY_CMD) $(LOADER_ELF) $(LOADER_BIN)
	$(call color_progress_prefix, "Name")
	$(DOCKER_MINIMAL_CMD) echo $(LOADER_BIN)
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

# ---------------------------------------------------------------------------- #
#                          Generate the documentation                          #
# ---------------------------------------------------------------------------- #
doc:
ifeq ($(DOCKER),y)
	$(call color_header, "Generating docs")
	$(DOCKER_MINIMAL_CMD) $(DOC_CMD)
else
	$(call color_header, "Generating docs")
	$(DOC_CMD)
endif

# ---------------------------------------------------------------------------- #
#                          Run the bootloader in QEMU                          #
# ---------------------------------------------------------------------------- #
# -- This is no longer used due to the need for tftp (likely to be removed) -- #
qemu: $(LOADER_BIN)
ifeq ($(DOCKER),y)
	$(call color_header, "Launching QEMU")
	$(DOCKER_MINIMAL_CMD) $(EXEC_QEMU) $(QEMU_ARGS)
else
	$(call color_header, "Launching QEMU")
	$(EXEC_QEMU) $(QEMU_ARGS)
endif

qemu_monitor: $(LOADER_BIN)
	$(call color_header, "Launching QEMU")
	$(EXEC_QEMU) $(QEMU_ARGS)

qemu_halted: $(LOADER_BIN)
	$(call color_header, "Launching QEMU")
	$(EXEC_QEMU) $(QEMU_ARGS) -S

# ---------------------------------------------------------------------------- #
#                                  Run clippy                                  #
# ---------------------------------------------------------------------------- #
clippy:
ifeq ($(DOCKER),y)
	$(DOCKER_MINIMAL_CMD) $(CLIPPY_CMD)
else
	$(CLIPPY_CMD)
endif

# ---------------------------------------------------------------------------- #
#                                     Clean                                    #
# ---------------------------------------------------------------------------- #
clean:
ifeq ($(DOCKER),y)
	$(DOCKER_MINIMAL_CMD) rm -rf target $(LOADER_BIN) bootloader.ld
else
	rm -rf target $(LOADER_BIN) bootloader.ld
endif

# ---------------------------------------------------------------------------- #
#                                  Run readelf                                 #
# ---------------------------------------------------------------------------- #
readelf: $(LOADER_ELF)
	$(call color_header, "Launching readelf")
	$(READELF_BINARY) --headers $(LOADER_ELF)

# ---------------------------------------------------------------------------- #
#                                  Run objdump                                 #
# ---------------------------------------------------------------------------- #
objdump: $(LOADER_ELF)
	$(call color_header, "Launching objdump")
	@$(DOCKER_TOOLS) $(OBJDUMP_BINARY) --disassemble --demangle \
                --section .text     \
                --section .rodata   \
                $(LOADER_ELF) | rustfilt

# ---------------------------------------------------------------------------- #
#                                    Run nm                                    #
# ---------------------------------------------------------------------------- #
nm: $(LOADER_ELF)
	$(call color_header, "Launching nm")
	$(NM_BINARY) --demangle --print-size $(LOADER_ELF) | sort | rustfilt

# ---------------------------------------------------------------------------- #
#                                   Run tests                                  #
# ---------------------------------------------------------------------------- #
# -- This is no longer used due to the need for tftp (likely to be removed) -- #
test: $(LOADER_BIN)
	timeout 5m tftp/qemu_test.sh

# ---------------------------------------------------------------------------- #
#                           Generate call stack graph                          #
# ---------------------------------------------------------------------------- #
call_stack:
ifeq ($(DOCKER),y)
	$(DOCKER_FULL_CMD) cargo +nightly call-stack --bin bootloader --features \
	$(BSP) --target riscv64gc-unknown-none-elf > cg.dot ; \
	dot -Tsvg cg.dot > cg.svg && rm cg.dot
else
	cargo +nightly call-stack --bin bootloader --features $(BSP) --target \
	riscv64gc-unknown-none-elf > cg.dot ; \
	dot -Tsvg cg.dot > cg.svg && rm cg.dot
endif

# ---------------------------------------------------------------------------- #
#                             Execute cargo geiger                             #
# ---------------------------------------------------------------------------- #
geiger:
	echo "# Safety Report" > .github/workflows/geiger.md
ifeq ($(DOCKER),y)
	$(DOCKER_FULL_CMD) cargo geiger \
		--target riscv64gc-unknown-none-elf \
		--features $(BSP) --output-format GitHubMarkdown \
		--update-readme --readme-path .github/workflows/geiger.md
else
	cargo geiger --target riscv64gc-unknown-none-elf \
	--features $(BSP) --output-format GitHubMarkdown --update-readme \
	--readme-path .github/workflows/geiger.md
endif
	cat .github/workflows/geiger.md

# ---------------------------------------------------------------------------- #
#                               Execute hyperfine                              #
# ---------------------------------------------------------------------------- #
# -- This is no longer used due to the need for tftp (likely to be removed) -- #
hyperfine:
ifeq ($(DOCKER),y)
	$(DOCKER_FULL_CMD) touch test.md
	$(DOCKER_FULL_CMD) hyperfine --warmup 1 --show-output \
		--export-markdown test.md ./tftp/qemu_test.sh
	$(DOCKER_FULL_CMD) cat test.md
else
	touch test.md
	hyperfine --warmup 1 --show-output --export-markdown test.md \
		./tftp/qemu_test.sh
	cat test.md
endif

# ---------------------------------------------------------------------------- #
#                             Execute cargo expand                             #
# ---------------------------------------------------------------------------- #
expand:
ifeq ($(DOCKER),y)
	$(DOCKER_FULL_CMD) cargo expand \
		--target riscv64gc-unknown-none-elf --features $(BSP)
else
	cargo expand --target riscv64gc-unknown-none-elf --features $(BSP)
endif
