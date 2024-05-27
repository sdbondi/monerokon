To build:

```
cd exercise-4
cargo build-wasm
```

To test:
```
cargo test
```

build-wasm is defined in `.cargo/config.toml`:

```
[alias]
build-wasm = "build --target=wasm32-unknown-unknown"
```
