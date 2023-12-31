# QuorumForge — build orchestration for the Rust engine and TypeScript viewer.
#
# Common targets:
#   make build     compile the release engine and the viewer
#   make test      run every test suite in both languages
#   make demo      run a full pipeline against the sample deliberations
#   make bundle    build and verify a deterministic evidence bundle
#   make fmt       format Rust sources (requires rustfmt)
#   make clean     remove build artifacts

CARGO ?= cargo
NODE  ?= node
NPM   ?= npm

SAMPLE_QF   := samples/cache-coherence.qf
SAMPLE_JSON := samples/migration-strategy.json

.DEFAULT_GOAL := help

.PHONY: help
help:
	@echo "QuorumForge make targets:"
	@echo "  build         build the release engine and the viewer"
	@echo "  build-rust    build the Rust engine (release)"
	@echo "  build-viewer  install + compile the TypeScript viewer"
	@echo "  test          run all Rust and viewer tests"
	@echo "  test-rust     run the Rust test suite (incl. doctests)"
	@echo "  test-viewer   run the viewer test suite"
	@echo "  demo          adjudicate the samples and render via the viewer"
	@echo "  bundle        build and verify a deterministic bundle"
	@echo "  fmt           format Rust sources"
	@echo "  clean         remove build artifacts"

.PHONY: build
build: build-rust build-viewer

.PHONY: build-rust
build-rust:
	$(CARGO) build --release

.PHONY: build-viewer
build-viewer:
	cd viewer && $(NPM) install --no-audit --no-fund && $(NPM) run build

