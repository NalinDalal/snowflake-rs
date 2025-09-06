# id-gnrt-rust-impl

A Rust implementation of the Snowflake ID generator algorithm. This project provides a fast, thread-safe way to generate unique, time-ordered 64-bit IDs, inspired by Twitter's Snowflake.

[official link](https://crates.io/crates/id-gnrt-rust-impl)

## Features

- Generates unique 64-bit IDs based on timestamp, datacenter, machine, and sequence.
- Thread-safe and efficient.
- Custom epoch for compact IDs.

## Usage

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
id-gnrt-rust-impl = "0.1.0"
```

Example usage:

```rust
use id_gnrt_rust_impl::Snowflake;

let generator = Snowflake::new(1, 1);
let id = generator.next_id();
println!("Generated ID: {}", id);
```

## License

MIT
