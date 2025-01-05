# Default configuration variables
TARGET ?= aarch64-unknown-none
BUILD_TYPE ?= debug
PROJECT_NAME ?= bk
CC = aarch64-none-elf-gcc
AR = aarch64-none-elf-ar
AS = aarch64-none-elf-as

# Derived variables
TARGET_DIR := target/$(TARGET)/$(BUILD_TYPE)
ELF_FILE := $(TARGET_DIR)/$(PROJECT_NAME)
BIN_FILE := $(TARGET_DIR)/$(PROJECT_NAME).bin
DISASSEMBLY_FILE := $(TARGET_DIR)/disassembled.txt

# Tools
CARGO := cargo
OBJCOPY := rust-objcopy
OBJDUMP := aarch64-none-elf-objdump
SIZE := aarch64-none-elf-size
QEMU := qemu-system-aarch64

# Flags
CARGO_FLAGS := $(if $(filter $(BUILD_TYPE),release),--release,)
OBJCOPY_FLAGS := --binary-architecture aarch64 -O binary
OBJDUMP_FLAGS := -D
QEMU_FLAGS := -M raspi4b -cpu cortex-a72 -nographic -kernel $(BIN_FILE) -s -S

# Default target
all: build

# Build target
build:
	export CC=$(CC) && \
	export AR=$(AR) && \
		$(CARGO) build --target $(TARGET) $(CARGO_FLAGS)
	$(OBJCOPY) $(OBJCOPY_FLAGS) $(ELF_FILE) $(BIN_FILE)

# Clean target
clean:
	$(CARGO) clean

# Run target
run: build
	$(QEMU) $(QEMU_FLAGS)

# Disassemble target
disassemble: build
	$(OBJDUMP) $(OBJDUMP_FLAGS) $(ELF_FILE) > $(DISASSEMBLY_FILE)
	@echo "Disassembly written to $(DISASSEMBLY_FILE)"

size: build
	$(SIZE) $(ELF_FILE)

# Format code
fmt:
	$(CARGO) fmt

# Lint code
lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

# Test
test:
	$(CARGO) test --target $(TARGET) $(CARGO_FLAGS)

# Help target
help:
	@echo "Usage: make [TARGET]"
	@echo "Available targets:"
	@echo "  build      Build the project (default target)"
	@echo "  clean      Clean build artifacts"
	@echo "  run        Run the binary in QEMU (Raspberry Pi 4 model)"
	@echo "  fmt        Format the codebase"
	@echo "  lint       Lint the codebase with Clippy"
	@echo "  test       Run tests"
	@echo "  help       Display this help message"

# Phony targets
.PHONY: all build clean run fmt lint test help
