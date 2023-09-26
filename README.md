# stac-server-rs

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/gadomski/stac-server-rs/ci.yaml?branch=main&style=for-the-badge)](https://github.com/gadomski/stac-server-rs/actions/workflows/ci.yaml)
[![STAC API Validator](https://img.shields.io/github/actions/workflow/status/gadomski/stac-server-rs/validate.yaml?branch=main&label=STAC+API+Validator&style=for-the-badge)](https://github.com/gadomski/stac-server-rs/actions/workflows/validate.yaml)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

A [STAC API](https://github.com/radiantearth/stac-api-spec) written in Rust.

| Crate | Description |
| ----- | ---- |
| [stac-api-backend](./stac-api-backend/README.md) | Generic backend interface for STAC APIs |
| [stac-server](./stac-server/README.md) | A STAC API server in [axum](https://github.com/tokio-rs/axum) |
| [stac-server-cli](./stac-server-cli/README.md) | A command-line interface for [stac-server](./stac-server/README.md) |

## Usage

You'll need [rust](https://rustup.rs/).
Then:

```shell
cargo install stac-server-cli
```

Any collections, items, or item collections provided on the command line will be ingested into the backend on startup.
To start a memory-backed server populated with one collection and one item from the [Planetary Computer](https://planetarycomputer.microsoft.com/):

```shell
stac-server \
    https://planetarycomputer.microsoft.com/api/stac/v1/collections/3dep-seamless \
    https://planetarycomputer.microsoft.com/api/stac/v1/collections/3dep-seamless/items/n34w116-13
```

If you have a [pgstac](https://github.com/stac-utils/pgstac) database pre-populated with collections and items, you can point your server there:

```shell
stac-server --pgstac postgres://username:password@localhost/postgis
```

For more advanced setups, use a [configuration file](#configuration):

```shell
stac-server --config config.toml
```

## Configuration

The [`Config` structure](https://docs.rs/stac-server/latest/stac-server-cli/struct.Config.html) defines the configuration attributes available for your server.
This repository includes [a default configuration](./stac-server-cli/src/config.toml) that you can then customize for your use-case.

## Conformance classes

The STAC API spec uses "conformance classes" to describe the functionality of a server.
These are the supported conformance classes for each backend:

| Conformance class | Memory backend | pgstac backend |
| -- | -- | -- |
| [Core](https://github.com/radiantearth/stac-api-spec/tree/main/core) | ✅ | ✅ |
| [Features](https://github.com/radiantearth/stac-api-spec/tree/main/ogcapi-features) | ✅ | ✅ |
| [Item search](https://github.com/radiantearth/stac-api-spec/tree/main/item-search) | ❌ | ❌ |

## Testing

In addition to unit tests, **stac-server** comes with some integration tests for both the memory and **pgstac** backends.
The **pgstac** test is ignored by default, since it requires a running **pgstac** database.
To run the **pgstac** integration test:

```shell
docker-compose up -d
cargo test -- --ignored
docker-compose down
```

## Validation

Conformance classes are validated with [stac-api-validator](https://github.com/stac-utils/stac-api-validator) in [CI](https://github.com/gadomski/stac-server-rs/actions/workflows/validate.yaml).
To validate yourself, you'll need to install **stac-api-validator**, preferably in a virtual enviroment:

```shell
pip install stac-api-validator
```

Then, with the memory backend:

```shell
scripts/validate
```

To validate the server with the pgstac backend, you'll need to start a pgstac server first:

```shell
docker-compose up -d
scrips/validate --pgstac
docker-compose down
```

### Conda

If you like [conda](https://docs.conda.io), you can use our provided environment to validate:

```shell
conda env create -f scripts/environment.yaml
conda activate validate
scripts/validate
```

## License

**stac-server-rs** is dual-licensed under both the MIT license and the Apache license (Version 2.0).
See [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) for details.
