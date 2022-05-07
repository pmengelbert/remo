.PHONY: build_arm scp

tar:
	cargo run

build_arm:
	docker buildx build --output=type=local,dest=$(shell pwd)/bin/ --platform=linux/arm64 -t doesnot:matter .

scp:
	scp bin/remo p:
