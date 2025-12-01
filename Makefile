.DELETE_ON_ERROR:
.ONESHELL:
.POSIX:

DEVPORT ?= 8080
ASSET_DIRECTORY ?= $(shell pwd)/uploads
DATABASE_FILE ?= database.db
DATABASE_URL ?= $(shell pwd)/$(DATABASE_FILE)
STATIC_DIRECTORY ?= $(shell pwd)/../client
NOT_FOUND_PAGE ?= $(shell pwd)/../client_old/index.html
BUILD_FLAGS ?=
TWITCH_REDIRECT_URI ?= http://localhost:$(DEVPORT)/callback/twitch # Must be localhost, twitch doesn't allow plain http on 127.0.0.1

MIGRATIONS := $(shell find migrations/ -type f -name '*.sql')
SRC_FILES := $(shell find src/ -type f) Cargo.toml Cargo.lock Makefile
RUNTIME_ENV := ASSET_DIRECTORY=$(ASSET_DIRECTORY) \
               DATABASE_URL=$(DATABASE_URL) \
			   TWITCH_REDIRECT_URI=$(TWITCH_REDIRECT_URI)

.PHONY: all
all: build

.PHONY: build
build: $(MIGRATIONS) $(SRC_FILES)
	cargo build $(BUILD_FLAGS)

.PHONY: test
test:
	cargo test

.PHONY: run
run: database.db
	diesel migration run --database-url $(DATABASE_URL)
	mkdir -p $(ASSET_DIRECTORY)
	$(RUNTIME_ENV) dx serve --hot-reload true --features web --port $(DEVPORT)

.PHONY: fix
fix:
	cargo fix --allow-dirty

database.db: $(MIGRATIONS)
	diesel setup --database-url $(DATABASE_URL)
