#
# Makefile
#

BIN_DIR := target/debug/deps
COVERAGE_DIR := target/coverage

nothing:

coverage:
	@rm -rf $(COVERAGE_DIR)
	CARGO_INCREMENTAL=0 \
	  LLVM_PROFILE_FILE=cargo-test-%p-%m.profraw \
	  RUSTDOCFLAGS="-C instrument-coverage -Z unstable-options --persist-doctests $(BIN_DIR)" \
	  RUSTFLAGS="-C instrument-coverage" \
    	  cargo +nightly test
	@grcov --binary-path $(BIN_DIR) -s . -t html --branch --ignore-not-existing \
	  -o $(COVERAGE_DIR)/html .
	@# Remove `.profraw` files
	@rm $$(find -name "cargo-test-*.profraw")
	@# Remove doc-test executables
	@rm -rfv $(BIN_DIR)/src_*_rs_*
	@echo Created $(COVERAGE_DIR)/html/index.html
	@echo Done.

toolchain:
	@cargo r --bin meadows-env | grep ^RUSTUP_TOOLCHAIN
	@strings target/debug/meadows-env | grep -o '^/rustc/[^/]\+/' | uniq

# EOF
