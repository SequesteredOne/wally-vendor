set windows-shell := ["powershell.exe", "-c"]

build:
    cargo build

install: build
    cargo install --path . --force

run *args:
    cargo run -- {{args}}

    