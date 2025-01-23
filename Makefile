#
# Makefile
#

BIN_DIR := target/debug/deps
COVERAGE_DIR := target/coverage

nothing:

bench:
	@cargo +nightly bench

clippy:
	@cargo +nightly clippy --all-features --all-targets

coverage:
	@rm -rf $(COVERAGE_DIR)
	CARGO_INCREMENTAL=0 \
	  LLVM_PROFILE_FILE=cargo-test-%p-%m.profraw \
	  RUSTDOCFLAGS="-C instrument-coverage -Z unstable-options --persist-doctests $(BIN_DIR)" \
	  RUSTFLAGS="-C instrument-coverage" \
    	  cargo +nightly test
	@grcov --binary-path $(BIN_DIR) -s . -t html --branch --ignore-not-existing -o $(COVERAGE_DIR)/html .
	@# Remove `.profraw` files
	@rm -v $$(find -name "cargo-test-*.profraw")
	@# Remove doc-test executables
	@rm -rfv $(BIN_DIR)/src_*_rs_*
	@echo Created $(COVERAGE_DIR)/html/index.html

doc:
	@cargo doc --all-features

fmt-check:
	@cargo +nightly fmt --check
	
miri-run:
	@cargo +nightly miri run

miri-test:
	@cargo +nightly miri test

msrv:
	@cargo msrv find

# Pre-publish actions. All should be well: No warnings, etc.
publish: fmt-check clippy doc
	@cargo test

test-doc:
	@cargo test --doc -- --show-output

# EOF
