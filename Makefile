.PHONY: build_arm scp

tar:
	cargo run

build_arm:
	cross build --release --target=armv7-unknown-linux-musleabihf

scp:
	scp bin/remo p:bin/remo

build_scp: build_arm scp
