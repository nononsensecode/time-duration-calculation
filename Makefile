# Makefile for building Linux and Windows executables for time_duration_calculator

BINARY_NAME = time_duration_calculator
LINUX_TARGET = x86_64-unknown-linux-gnu
WINDOWS_TARGET = x86_64-pc-windows-gnu

.PHONY: build clean

build:
	cargo build --release --target $(LINUX_TARGET)
	cargo build --release --target $(WINDOWS_TARGET)
	@echo "Linux binary: target/$(LINUX_TARGET)/release/$(BINARY_NAME)"
	@echo "Windows binary: target/$(WINDOWS_TARGET)/release/$(BINARY_NAME).exe"

clean:
	cargo clean
