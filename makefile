RFLAGS="-C link-arg=-s"
SRC_DIR=contracts/ref-ve

build: contracts/ref-ve
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p ref-ve --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/ref_ve.wasm ./res/ref_ve.wasm

test: build mock-ft mock-mft
	RUSTFLAGS=$(RFLAGS) cargo test -p ref-ve 

release:
	$(call docker_build,_rust_setup.sh)
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/ref_ve.wasm res/ref_ve_release.wasm

TEST_FILE ?= **
LOGS ?=
sandbox: build mock-ft mock-mft
	mkdir -p sandbox/compiled-contracts/
	cp res/*.wasm sandbox/compiled-contracts/
	cd sandbox && \
	NEAR_PRINT_LOGS=$(LOGS) npx near-workspaces-ava --timeout=5m __tests__/ref-ve/$(TEST_FILE).ava.ts --verbose

mock-ft: contracts/mock-ft
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock-ft --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_ft.wasm ./res/mock_ft.wasm

mock-mft: contracts/mock-mft
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock-mft --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_mft.wasm ./res/mock_mft.wasm

clean:
	cargo clean
	rm -rf res/

define docker_build
	docker build -t my-contract-builder .
	docker run \
		--mount type=bind,source=${PWD},target=/host \
		--cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
		-w /host \
		-e RUSTFLAGS=$(RFLAGS) \
		-i -t my-contract-builder \
		/bin/bash $(1)
endef
