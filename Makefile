# Set this to any non-empty string to enable unoptimized
# build w/ debugging features.
debug ?=

# All complication artifacts, including dependencies and intermediates
# will be stored here, for all architectures.  Use a non-default name
# since the (default) 'target' is used/referenced ambiguously in many
# places in the tool-chain (including 'make' itself).
CARGO_TARGET_DIR ?= target
export CARGO_TARGET_DIR  # 'cargo' is sensitive to this env. var. value.

ifdef debug
$(info debug is $(debug))
  # These affect both $(CARGO_TARGET_DIR) layout and contents
  # Ref: https://doc.rust-lang.org/cargo/guide/build-cache.html
  release :=
  profile :=debug
else
  release :=--release
  profile :=release
endif

# Run all builds on 4 cpus
build-release:
	cargo build --release


.PHONY: all
all: build

bin:
	mkdir -p $@

$(CARGO_TARGET_DIR):
	mkdir -p $@


.PHONY: build
build: bin $(CARGO_TARGET_DIR)
	cargo build  $(release)
	cp $(CARGO_TARGET_DIR)/$(profile)/server bin/netavark-proxy$(if $(debug),.debug,)
	cp $(CARGO_TARGET_DIR)/$(profile)/client bin/client$(if $(debug),.debug,)
	sudo cp -R $(CARGO_TARGET_DIR)/$(profile)/server /usr/local/bin/netavark-proxy$(if $(debug),.debug,)

clean:
	rm -fr bin
	cargo clean
	rm -f proto-build/netavark_proxy.rs && sudo rm -f /usr/local/bin/netavark-dhcp-proxy

.PHONY: test
test: unit integration

.PHONY: unit
unit: $(CARGO_TARGET_DIR)
	cargo test

.PHONY: integration
integration: $(CARGO_TARGET_DIR)
	bats test/

.PHONY: validate
validate: $(VARGO_TARGET_DIR)
	cargo fmt --all -- --check
	cargo clippy --no-deps --fix --allow-dirty -- 
		
