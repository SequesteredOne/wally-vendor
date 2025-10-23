# wally-vendor

A fast and simple utility for vendoring [Wally](https://github.com/UpliftGames/wally)-installed dependencies anywhere in your project. This tool will copy packages from any specified realm (`Packages/`, `ServerPackages/`, `DevPackages/`, or auto-detected) into their own vendor directories (or throw them all into one directory).

*__Note:__ This tool is not an official affiliate of [Wally](https://github.com/UpliftGames/wally). It is, however, meant to interact with the outputs of Wally.*

## Common Usage

*(For a full list of options, please use `wally-vendor --help`, or `wally-vendor <COMMAND> --help`)*

Nagivate to your project directory (the one containing `wally.toml`) and run `sync` command:

```bash
wally-vendor sync
```

This will read your `wally.toml` file, find the installed packages, and copy them into a `WallyVendor` directory by default.

## Advanced Usage

### Custom Vendor Directories

To change the default vendor directory, you can pass command line arguments:

```bash
wally-vendor sync --shared-dir "path/to/shared" --server-dir "path/to/server" --dev-dir "path/to/dev"
```

Or use a single directory for all dependencies:

```bash
wally-vendor sync --vendor-dir "path/to/all"
```

### Configuration File

To avoid passing arguments every run, you can add a `[wally-vendor]` section to your `wally.toml`:

```toml
[wally-vendor]
shared-dir = "path/to/shared"
server-dir = "path/to/server"
dev-dir = "path/to/dev"
```

### Using a Separate Configuration File

If you want to vendor only specific dependencies, you can create a `wally-vendor.toml` file that mimics the structure of `wally.toml`:

```toml
[wally-vendor]
shared-dir = "src/shared/SharedPackages"
server-dir = "src/server/ServerPackages"
dev-dir = "src/shared/DevPackages"

[dependencies]
promise = "evaera/promise@4.0.0"
janitor = "howmanysmall/janitor@1.18.3"

[server-dependencies]
profilestore = "lm-loleris/profilestore@1.0.3"

[dev-dependencies]
jest = "jsdotlua/jest@3.10.0"
```

If both `wally.toml` and `wally-vendor.toml` exist, the tool will prioritize `wally-vendor.toml`.

#### Additional Options

- Clean vendor directories before syncing:

```bash
wally-vendor sync --clean
```

- Fail if any required dependency is missing:

```bash
wally-vendor sync --strict
```

- Limit to specific realms:

```bash
wally-vendor sync --realm server --realm shared
```

(This will not sync the dev realm)

#### Features

1. Uses [rayon](https://github.com/rayon-rs/rayon) for parallel processing when copying dependencies. ([tokio](https://github.com/tokio-rs/tokio) was considered, and tested but provided slower benchmarks. A branch containing the tokio implementation can be found, incase it could be further optimized.)

2. Avoids re-copying files that haven't changed for incremental updates.

3. Reads `wally.lock` (lockfile) to vendor exact package versions for deterministic outputs. (Will proceed anyways without a lockfile, unless the `--locked` flag is provided)
