# stac-server-rs

A [STAC API](https://github.com/radiantearth/stac-api-spec) written in Rust.
Currently _very_ experimental and un-featured.

## Usage

You'll need [rust](https://rustup.rs/).
Then:

```shell
cargo install --git https://github.com/gadomski/stac-server-rs
```

You'll need a simple configuration file, use [the example configuration](./data/config.toml) as a starting point.
To start a simple memory-backed server, loaded in with collections, items, and item collections at local paths:

```shell
stac-server config.toml memory collection.json item-collection.json
```

### pgstac

A [pgstac](https://github.com/stac-utils/pgstac) backend is provided:

```shell
stac-server config.toml pgstac postgres://username:password@localhost/postgis
```

If you need a **pgstac** database with a bunch of collections and items, may we recommend [pc-mini](https://github.com/gadomski/pc-mini).

## API

We tried our best to separate responsibilities, so there's a couple of crates in this repo that compose together to make the command-line server:

- [stac-backend](./stac-backend/) defines the interface that all backends will implement, and provides the simple `MemoryBackend`
- [pgstac](./pgstac/) provides a **pgstac** backend
- [stac-server](./stac-server/) is the server itself
- [stac-server-cli](./stac-server-cli/) wraps everything together into an executable

This hopefully will make it easy to, e.g., implement other backends, or use the server api in a different application (e.g. as part of another service).
