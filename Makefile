
.PHONY: dev
dev: 
	cargo build && sudo ./target/debug/dsl_restart

.PHONY: build
build: 
	cross build --target armv7-unknown-linux-gnueabi --release

.PHONY: clean
clean: 
	cargo clean