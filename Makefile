# Makefile for building Linux and Windows executables for time_duration_calculator

BINARY_NAME = time_duration_calculator
LINUX_TARGET = x86_64-unknown-linux-gnu
WINDOWS_TARGET = x86_64-pc-windows-gnu
MAC_TARGET = x86_64-apple-darwin
MAC_ARM_TARGET = aarch64-apple-darwin

.PHONY: build clean

build:
	cargo build --release --target $(LINUX_TARGET)
	cargo build --release --target $(WINDOWS_TARGET)
	cargo build --release --target $(MAC_TARGET)
	cargo build --release --target $(MAC_ARM_TARGET)
	@echo "Linux binary: target/$(LINUX_TARGET)/release/$(BINARY_NAME)"
	@echo "Windows binary: target/$(WINDOWS_TARGET)/release/$(BINARY_NAME).exe"
	@echo "Mac (Intel) binary: target/$(MAC_TARGET)/release/$(BINARY_NAME)"
	@echo "Mac (Apple Silicon) binary: target/$(MAC_ARM_TARGET)/release/$(BINARY_NAME)"

clean:
	cargo clean
