TARGET := riscv64gc-unknown-none-elf
MODE := release

KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
FS_IMG := ../user/target/$(TARGET)/$(MODE)/fs.img
APPS := ../user/src/bin/*

# kernel entry
KERNEL_ENTRY_PA := 0x80000000

# board
BOARD := qemu
SBI ?= rustsbi

# bootloader
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# Run usertests or usershell
TEST ?= 

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

build: env $(KERNEL_BIN) fs-img

$(KERNEL_BIN): kernel
	@$(OBJCOPY) --strip-all $(KERNEL_ELF) -O binary $@

fs-img: $(APPS)
	@cd ../user && make build TEST=$(TEST)
	@rm -rf $(FS_IMG)
	@cd ../easy-fs-fuse && cargo run --$(MODE) -- -s=../user/src/bin/ -d=../user/target/$(TARGET)/$(MODE)/

$(APPS):
	@echo $(APPS)

kernel: 
	@cd ../user && make build
	@cp src/linker-$(BOARD).ld src/linker.ld
	@cargo build $(MODE_ARG)
	@rm src/linker.ld

run: build
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios none \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
		-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0
	
debug: build
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -bios none -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -s -S" && \
		tmux split-window -h "gdb-multiarch -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

# remove ./target directory
clean:
	@cargo clean

.PHONY: build env kernel clean run-inner fs-img