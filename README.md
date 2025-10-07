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
cargo codspeed build
cargo codspeed run
```

## Benchmarks

This repository includes a simple "Hello, World!" benchmark using [divan](https://docs.rs/divan/) via the CodSpeed compatibility layer.
