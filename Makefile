.PHONY: build_arm scp

tar:
	cargo run

build_arm:
	cross build --release --target=armv7-unknown-linux-musleabihf
	cp target/armv7-unknown-linux-musleabihf/release/remo bin/remo

scp:
	scp bin/remo p:bin/remo

build_scp: build_arm scp
