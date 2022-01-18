build:
	cargo fmt
	cargo build

build-release:
	cargo fmt
	cargo build --release

test:
	cargo fmt
	cargo test --all

run:
	cargo fmt
	cargo run

install:
	cargo fmt
	cargo install --path .

.PHONY: build test run install