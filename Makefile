all: build

build:
	cargo build

install: install-certgen install-tcp-echo install-tcp-client install-tcp-fibonacci install-katey-el-es install-katey-client

install-certgen:
	cargo install --path certgen

install-tcp-echo:
	cargo install --path tcp-echo

install-tcp-client:
	cargo install --path tcp-client

install-tcp-fibonacci:
	cargo install --path tcp-fibonacci

install-katey-el-es:
	cargo install --path katey-el-es

install-katey-client:
	cargo install --path katey-client

update:
	cargo update

test:
	cargo test

manual-test:
	(cd manual-test && PATH=$(CURDIR)/target/debug:$(PATH) decompose fixture.toml)

check:
	cargo check --bins --examples --tests

format:
	cargo fmt

clean:
	cargo clean

.PHONY: all build test update check unit-test clean install-certgen install-tcp-echo install-tcp-fibonacci install-tcp-client install-katey-el-es install-katey-client manual-test
