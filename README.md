# stac-server-rs

A simple STAC server written in Rust.

## Installation

Install rust.
We recommend [rustup](https://rustup.rs/).
Then, install **stac-server-rs**:

```shell
cargo install --git http://github.com/gadomski/stac-server-rs
```

## Usage

To run a simple memory-backed server, initialized with a single collection:

```shell
stac-server 0.0.0.0:3000 example-config.toml --href data/joplin/collection.json simple
```

### pgstac

**stac-server-rs** currently comes with two backends: a simple in-memory store, and [pgstac](https://github.com/stac-utils/pgstac).
Our [docker-compose](./docker-compose.yml) file provides a simple way to run **stac-server-rs** against a local **pgstac** instance, loaded with some simple [test data](data//joplin/):

```shell
docker-compose up
```

You can then browse the **pgstac**-backed server at <http://localhost:3000>.

To point the server at an existing **pgstac** instance, check the arguments for the `pgstac` subcommand:

```shell
stac-server pgstac --help
```
