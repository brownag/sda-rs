.PHONY: help install build test lint format check run clean all

SHELL := /bin/bash
.DEFAULT_GOAL := help

CARGO ?= cargo
PORT  ?= 8080
DB    ?= sda.gpkg

help: ## Show this help message
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / { printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

install: ## Fetch dependencies
	$(CARGO) fetch

build: ## Build the project
	$(CARGO) build

test: ## Run all tests
	$(CARGO) test

lint: ## Run clippy and check formatting
	$(CARGO) fmt --check
	$(CARGO) clippy --all-targets

format: ## Format code
	$(CARGO) fmt

check: lint test ## Run lint and tests

run: ## Run the server (PORT=, DB= to override)
	$(CARGO) run -- --backend-url $(DB) --port $(PORT)

clean: ## Clean build artifacts
	$(CARGO) clean

all: check build ## Run full pipeline (check + build)
