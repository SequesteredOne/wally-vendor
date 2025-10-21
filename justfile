WALLY_VENDOR_CMD := 'wally-vendor sync-vendor --realm server --realm shared --realm dev --server-dir VendorServer --shared-dir VendorShared --dev-dir VendorDev --strict'

set windows-shell := ["powershell.exe", "-c"]

build:
    cargo build

install: build
    cargo install --path . --force

run *args:
    cargo run -- {{args}}

test: install
    cd tests/example; wally install; wally-vendor sync-vendor --realm server --realm shared --realm dev --clean

extreme-test: install
    cd tests/extreme-example; wally install; wally-vendor sync-vendor --realm server --realm shared --realm dev --server-dir VendorServer --shared-dir VendorShared --dev-dir VendorDev --clean --strict

setup-benchmark: install
    cd tests/extreme-example; wally install

bench-clean: setup-benchmark
    cd tests/extreme-example; hyperfine --prepare '{{WALLY_VENDOR_CMD}}' --time-unit millisecond --export-markdown ../../benchmarks/clean.md 'wally-vendor sync-vendor --realm server --realm shared --realm dev --server-dir VendorServer --shared-dir VendorShared --dev-dir VendorDev --clean --strict'

bench-no-clean: setup-benchmark
    cd tests/extreme-example; wally-vendor sync-vendor --realm server --realm shared --realm dev --server-dir VendorServer --shared-dir VendorShared --dev-dir VendorDev --strict; hyperfine --prepare '{{WALLY_VENDOR_CMD}}' --time-unit millisecond --export-markdown ../../benchmarks/pre-vendored-no-clean.md 'wally-vendor sync-vendor --realm server --realm shared --realm dev --server-dir VendorServer --shared-dir VendorShared --dev-dir VendorDev --strict'
    