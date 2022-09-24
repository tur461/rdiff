.DEFAULT_GOAL := build

build:
	clear && cargo build --release

test:
	clear && cargo test


