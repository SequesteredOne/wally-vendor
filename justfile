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