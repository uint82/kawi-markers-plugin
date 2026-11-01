# Developing markers

## Requirements

- Rust 1.98 or newer
- The `wasm32-wasip2` target
- The `wasm-component-ld` linker
- Kawi with managed plugin support

```bash
rustup target add wasm32-wasip2
cargo install wasm-component-ld --version 0.5.31 --locked
```

## Build

```bash
./build.sh
```

If `kawi` is not installed globally:

```bash
KAWI_MANIFEST=/path/to/kawi/Cargo.toml ./build.sh
```

The script runs `kawi plugin build`, writes `markers.wasm`, and updates the `sha256` field in `plugin.toml`. The WASM file is not committed.

## Local development

```bash
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/kawi/plugins/markers"
cp plugin.toml markers.wasm "${XDG_CONFIG_HOME:-$HOME/.config}/kawi/plugins/markers/"
```

Inside Kawi:

```text
:plugin reload
:plugin list
:markers.scan
```

Development plugins load with a warning and are not managed by `:plugin install`, `:plugin update`, or `:plugin remove`.

## Release

Run the build first so `plugin.toml` contains the checksum of the component you are about to release.

```bash
./build.sh
git add Cargo.toml Cargo.lock plugin.toml src build.sh README.md DEVELOPING.md .gitignore
git commit -m "release v0.1.3"
git branch -M main
git remote add origin git@github.com:OWNER/kawi-markers-plugin.git
git push -u origin main
git tag v0.1.3
git push origin v0.1.3
gh release create v0.1.3 markers.wasm --title "v0.1.3" --notes "Blend the colon into the marker badge"
```

Replace `OWNER` with the GitHub account that owns the repository. The release must contain the `markers.wasm` asset. Kawi reads `plugin.toml` from the release commit and downloads the component from the release asset. The checksum in the committed manifest must match the released component.
