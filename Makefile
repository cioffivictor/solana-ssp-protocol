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
