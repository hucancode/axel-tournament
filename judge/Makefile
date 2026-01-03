.PHONY: pull-images clean

# Image names
SANDBOX_IMAGE ?= debian:trixie-slim
RUST_COMPILER_IMAGE ?= rust:slim-trixie
GO_COMPILER_IMAGE ?= golang:trixie
C_COMPILER_IMAGE ?= gcc:trixie

pull-images: rust-compiler.tar go-compiler.tar c-compiler.tar sandbox.tar

rust-compiler.tar:
	docker pull $(RUST_COMPILER_IMAGE)
	docker save $(RUST_COMPILER_IMAGE) -o rust-compiler.tar

go-compiler.tar:
	docker pull $(GO_COMPILER_IMAGE)
	docker save $(GO_COMPILER_IMAGE) -o go-compiler.tar

c-compiler.tar:
	docker pull $(C_COMPILER_IMAGE)
	docker save $(C_COMPILER_IMAGE) -o c-compiler.tar

sandbox.tar:
	docker pull $(SANDBOX_IMAGE)
	docker save $(SANDBOX_IMAGE) -o sandbox.tar

clean:
	rm -f *.tar
