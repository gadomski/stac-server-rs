# stac-server-rs

A toy STAC server backed by [pgstac](https://github.com/stac-utils/pgstac).
To run:

```shell
stac-server config.toml username password postgis localhost 5439
```

## pgstac

If you need a simple pgstac database, check out the Joplin example in the [stac-fastapi repo](https://github.com/stac-utils/stac-fastapi/blob/97b091127e41b24a600cdbc49466074562f554ae/docker-compose.yml#L99-L115):

```shell
git clone https://github.com/stac-utils/stac-fastapi && cd stac-fastapi
make loadjoplin-pgstac
make docker-run-pgstac
```
