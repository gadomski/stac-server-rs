# stac-server-rs

A [STAC API](https://github.com/radiantearth/stac-api-spec) written in Rust.
Currently _very_ experimental and un-featured.

## Usage

You'll need [rust](https://rustup.rs/).
Then:

```shell
cargo install --git https://github.com/gadomski/stac-server-rs
```

To start a simple memory-backed server:

```shell
stac-server
```

If you have collection and items you'd like to load into the memory backend, provide them at the command-line:

```shell
stac-server \
    https://planetarycomputer.microsoft.com/api/stac/v1/collections/3dep-seamless \
    https://planetarycomputer.microsoft.com/api/stac/v1/collections/3dep-seamless/items/n34w116-13
```

### pgstac

A [pgstac](https://github.com/stac-utils/pgstac) backend is provided:

```shell
stac-server --pgstac postgres://username:password@localhost/postgis
```

If you need a **pgstac** database with a bunch of collections and items, may we recommend [pc-mini](https://github.com/gadomski/pc-mini).

## API

We tried our best to separate responsibilities, so there's a couple of crates in this repo that compose together to make the command-line server:

- [stac-api](./stac-api/) defines API-specific structures and behaviors, e.g. backends and endpoints, without doing any actual server work
- [stac-server](./stac-server/) is the server itself, implemented using [axum](https://github.com/tokio-rs/axum)
- [stac-server-cli](./stac-server-cli/) wraps everything together into an executable

This hopefully will make it easy to, e.g., implement other backends, or use the server api in a different application (e.g. as part of another service).

## License

**stac-server-rs** is dual-licensed under both the MIT license and the Apache license (Version 2.0).
See [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) for details.
