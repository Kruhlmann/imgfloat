.DELETE_ON_ERROR:
.ONESHELL:
.POSIX:

BUILD_FLAGS ?= 

SRC_FILES := $(shell find src/ assets/ -type f) Cargo.toml Cargo.lock Makefile

.PHONY: all
all: build

.PHONY: run
run:
	dx serve

.PHONY: build
build: $(SRC_FILES)
	dx build $(BUILD_FLAGS)

.PHONY: test
test:
	cargo test

.PHONY: fix
fix:
	dx fmt
	cargo fix --allow-dirty
