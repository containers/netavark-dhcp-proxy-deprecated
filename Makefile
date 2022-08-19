
# Run all builds on 4 cpus
build-release:
	cargo build --release -j 4

build:
	cargo build -j 4

clean:
	cargo clean
	rm -f proto-build/netavark_proxy.rs

server:
	cargo run -j 4 --bin server

client:
	cargo run  -j 4 --bin client

help:
	@printf '%s\n' \
		'Make commands:'\
		' -- build-release'\
		' -- build'\
		' -- clean'\
		' -- server:  builds and runs the server binary' \
		' -- client: builds and runs the demo client'