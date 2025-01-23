.DELETE_ON_ERROR:
.ONESHELL:
.POSIX:

ASSET_DIRECTORY ?= $(shell pwd)/uploads
DATABASE_FILE ?= database.db
DATABASE_URL ?= $(shell pwd)/$(DATABASE_FILE)
STATIC_DIR ?= ./client
NOT_FOUND_PAGE ?= ./client/index.html
BUILD_FLAGS ?= 

MIGRATIONS := $(shell find migrations/ -type f -name '*.sql')
SRC_FILES := $(shell find src/ -type f) Cargo.toml Cargo.lock Makefile
RUN_ENVIRONMENT := ASSET_DIRECTORY=$(ASSET_DIRECTORY) \
				   DATABASE_URL=$(DATABASE_URL) \
				   STATIC_DIR=$(STATIC_DIR) \
				   NOT_FOUND_PAGE=$(NOT_FOUND_PAGE) 

.PHONY: all
all: build

.PHONY: build
build: $(MIGRATIONS) $(SRC_FILES)
	cargo build $(BUILD_FLAGS)

.PHONY: run
run: database.db
	diesel migration run --database-url $(DATABASE_URL)
	mkdir -p $(ASSET_DIRECTORY)
	cargo run

.PHONY: fix
fix:
	cargo fix --allow-dirty

database.db: $(MIGRATIONS)
	diesel setup --database-url $(DATABASE_URL)
