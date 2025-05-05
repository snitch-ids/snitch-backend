snitch backend
==============

## start development services **`localhost:8081`**
```shell
cd docker
docker-compose up redis-stack
```

## run the backend (debug) and provide the dotenv file as input.

```shell
RUST_BACKTRACE=1 RUST_LOG=debug  cargo run -- local-dev.env
```

## Prune redis database

e.g. for testing

```shell
redis-cli flushall
```
