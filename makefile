
default: build
build:
	cargo build
test:
	cargo run

ci:
	make-ci build $$(find src -name *.rs)
