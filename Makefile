all: build

build:
	cargo build

install: install-certgen install-tcp_echo install-tcp_client install-tcp_fibonacci

install-certgen:
	cargo install --path certgen

install-tcp_echo:
	cargo install --path tcp_echo

install-tcp_client:
	cargo install --path tcp_client

install-tcp_fibonacci:
	cargo install --path tcp_fibonacci

update:
	cargo update

test:
	cargo test

check:
	cargo check --bins --examples --tests

format:
	cargo fmt

clean:
	cargo clean

.PHONY: all build test update check unit-test clean install-certgen install-tcp_echo install-tcp_fibonacci install-tcp_client
