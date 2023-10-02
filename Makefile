# General options
ARCH ?= riscv64
TARGET := riscv64gc-unknown-none-elf
SMP ?= 1
FEATURES ?=
LOG ?= warn

# Utility definitions and functions
GREEN_C := \033[92;1m
CYAN_C := \033[96;1m
YELLOW_C := \033[93;1m
GRAY_C := \033[90m
WHITE_C := \033[37m
END_C := \033[0m

QEMU := qemu-system-$(ARCH)
OBJDUMP ?= rust-objdump -d --print-imm-hex --x86-asm-syntax=intel
OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)

# App options
A ?= server
APP ?= $(A)

APP_NAME := $(shell basename $(APP))
LD_SCRIPT := $(CURDIR)/axhal/linker.lds

OUT_DIR ?= target/$(TARGET)/release

OUT_ELF := $(OUT_DIR)/$(APP_NAME)
OUT_BIN := $(OUT_DIR)/$(APP_NAME).bin

RUSTFLAGS := -C link-arg=-T$(LD_SCRIPT) -C link-arg=-no-pie
export RUSTFLAGS

export AX_LOG=$(LOG)

all: build

build: $(OUT_BIN)

disasm: build
	$(OBJDUMP) $(OUT_ELF) | less

run: build justrun

justrun:
	@printf "    $(CYAN_C)Running$(END_C) on qemu...\n"
	$(QEMU) -m 128M -smp $(SMP) -machine virt \
		-bios default -kernel $(OUT_BIN) -nographic \
		-D qemu.log -d in_asm,int,mmu,pcall,cpu_reset,guest_errors

$(OUT_BIN): $(OUT_ELF)
	$(OBJCOPY) $(OUT_ELF) --strip-all -O binary $@

$(OUT_ELF): FORCE
	@printf "    $(GREEN_C)Building$(END_C) App: $(APP_NAME), Arch: riscv64, Platform: qemu-virt, App type: rust\n"
	cargo build --manifest-path $(APP)/Cargo.toml --release \
		--target $(TARGET) --target-dir $(CURDIR)/target $(FEATURES)

clean:
	@rm -rf ./target
	@rm -f ./qemu.log

FORCE:
	@:

.PHONY: all build disasm run justrun debug clippy fmt test test_no_fail_fast clean FORCE
