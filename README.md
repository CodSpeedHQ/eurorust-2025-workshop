# EuroRust 2025 Workshop

A template repository for learning Rust performance measurement with [CodSpeed](https://codspeed.io/).

## Getting Started

This workshop uses [divan](https://docs.rs/divan/) for benchmarking with CodSpeed's compatibility layer.

### Install cargo-codspeed

```sh
cargo install cargo-codspeed --locked
```

### Run benchmarks locally

```sh
# generate the reference files
cargo run --release --bin generate_fasta
cargo run --release --bin generate_blobs
```

Running with `divan`:

```sh
cargo bench --bench bfs # here we are only running the bfs benchmark
```

Running the benchmarks with `cargo-codspeed`:

```sh
cargo codspeed build -m walltime
cargo codspeed run -m walltime
```

Note: You can also set the `CODSPEED_RUNNER_MODE` environment variable to `walltime` to avoid passing `-m walltime` every time.
