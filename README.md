# stac-server-rs

A simple STAC server written in Rust.
To run an example server:

```shell
stac-server 0.0.0.0:3000 example-config.toml --href data/joplin/collection.json simple
```

## pgstac

If you need a simple pgstac database, check out the Joplin example in the [stac-fastapi repo](https://github.com/stac-utils/stac-fastapi/blob/97b091127e41b24a600cdbc49466074562f554ae/docker-compose.yml#L99-L115):

```shell
git clone https://github.com/stac-utils/stac-fastapi && cd stac-fastapi
make loadjoplin-pgstac
make docker-run-pgstac
stac-server 0.0.0.0:3000 example-config.toml pgstac username password postgis localhost 5439
```
