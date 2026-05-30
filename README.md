# srvcs-cardinality

The set-cardinality service of the srvcs.cloud distributed standard library.

Its single concern: **how many distinct values are in a list of integers?**
It counts the number of distinct integers, ignoring duplicates and order.

`srvcs-cardinality` is a **leaf**: it depends on no other service and makes no
network calls. All work is local.

```text
result = the number of distinct integers in values
         cardinality([1, 2, 2, 3]) = 3
```

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Service identity, concern, and dependency list |
| `POST` | `/` | Report the number of distinct integers in `values` |
| `GET` | `/healthz` `/readyz` `/metrics` `/openapi.json` | srvcs service standard surface |

```sh
curl -s -X POST localhost:8080/ -H 'content-type: application/json' -d '{"values": [1, 2, 2, 3]}'
# {"values":[1,2,2,3],"result":3}

curl -s -X POST localhost:8080/ -H 'content-type: application/json' -d '{"values": [4, 4, 4]}'
# {"values":[4,4,4],"result":1}
```

Responses:

- `200 {"values": [...], "result": <int>}` — evaluated. `result` is the number
  of distinct integers in `values`.
- `422 {"error": "values must be integers"}` — some element of `values` is not a
  JSON integer.

The result is always an `i64`. Order and repetition do not matter; an empty list
has cardinality `0`, and a list of all-identical integers has cardinality `1`.

## Dependencies

None. `srvcs-cardinality` is a leaf set-operation service. Because it owns its
own validation, it rejects any non-integer element directly with `422` rather
than forwarding to a dependency.

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `SRVCS_BIND_ADDR` | `0.0.0.0:8080` | Bind address |
| `SRVCS_ENV` | `development` | Environment label for logs |
| `RUST_LOG` | `info,tower_http=info` | Tracing filter |

## Local checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

See [`srvcs/platform`](https://github.com/srvcs/platform) for the shared
standard.

> Note: the `cargoHash` in `flake.nix` is inherited from the template and must be
> refreshed with a `nix build` before the Nix gates pass.
