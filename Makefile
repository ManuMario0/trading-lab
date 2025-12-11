# Auto-detect Homebrew Prefix (Apple Silicon vs Intel)
HOMEBREW_PREFIX := $(shell brew --prefix 2>/dev/null || echo "/usr/local")
export PKG_CONFIG_PATH := $(HOMEBREW_PREFIX)/lib/pkgconfig:$(PKG_CONFIG_PATH)

help:
	@echo "Available commands:"
	@echo "  make build         - Build all components (Rust, C++, Python setup)"
	@echo "  make test          - Run tests for all components"
	@echo "  make clean         - Clean all build artifacts"

build: build-rust build-cpp build-python

build-rust:
	@echo "Building Rust components..."
	cd execution-engine && cargo build --release
	cd broker-gateway && cargo build --release

build-cpp:
	@echo "Building C++ components..."
	mkdir -p strategy-lab/build && cd strategy-lab/build && cmake .. && make
	mkdir -p multiplexer/build && cd multiplexer/build && cmake .. && make

build-python:
	@echo "Setting up Python environments..."
	cd data-pipeline && pip3 install -e .
	cd supervisor-frontend && pip3 install -e .

test:
	cd execution-engine && cargo test
	cd broker-gateway && cargo test
	cd strategy-lab/build && ctest
	cd multiplexer/build && ctest
	cd data-pipeline && pytest3

clean:
	cd execution-engine && cargo clean
	cd broker-gateway && cargo clean
	rm -rf strategy-lab/build
	rm -rf multiplexer/build
