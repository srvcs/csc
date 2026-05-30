# srvcs-csc

The cosecant orchestrator of the srvcs.cloud distributed standard library.

Its single concern: **trigonometry: cosecant.** It owns the *control flow* —
composing two float primitives — but does no arithmetic of its own. It asks
[`srvcs-sin`](https://github.com/srvcs/sin) for the sine of the angle, then asks
[`srvcs-floatdivide`](https://github.com/srvcs/floatdivide) for the reciprocal
of that sine.

```
csc(value):
    s = sin(value)
    return floatdivide(1, s)   # 1 / sin(value)
```

The result is an `f64` — a JSON number that may be fractional. For example
`csc(1.5707963267948966) == 1.0` (sin of π/2 is `1`, and `1 / 1 == 1`).

Validation is not handled here. This service never calls `srvcs-isnumber`
directly; instead its dependencies validate their own operands, and any `422`
they raise is forwarded verbatim.

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Service identity, concern, and dependency list |
| `POST` | `/` | Compute the cosecant of `value` |
| `GET` | `/healthz` `/readyz` `/metrics` `/openapi.json` | srvcs service standard surface |

```sh
curl -s -X POST localhost:8080/ -H 'content-type: application/json' -d '{"value": 1.5707963267948966}'
# {"value":1.5707963267948966,"result":1.0}
```

Responses:

- `200 {"value": n, "result": n}` — evaluated; `result` is a float.
- `422` — a dependency rejected an input (forwarded verbatim).
- `500` — a reachable dependency returned a `200` without a numeric `result`
  (a contract violation).
- `503` — a dependency is unavailable.

## Dependencies

- [`srvcs-sin`](https://github.com/srvcs/sin)
- [`srvcs-floatdivide`](https://github.com/srvcs/floatdivide)

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `SRVCS_BIND_ADDR` | `0.0.0.0:8080` | Bind address |
| `SRVCS_SIN_URL` | `http://127.0.0.1:8090` | Base URL of `srvcs-sin` |
| `SRVCS_FLOATDIVIDE_URL` | `http://127.0.0.1:8091` | Base URL of `srvcs-floatdivide` |
| `SRVCS_ENV` | `development` | Environment label for logs |
| `RUST_LOG` | `info,tower_http=info` | Tracing filter |

## Local checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Orchestration tests stand up *computing* mock dependency services in-process —
they read the request body and return the real `sin(value)` / `a / b`, so the
composition is genuinely exercised against the asserted cases (compared
approximately, since the result is a float). See
[`srvcs/platform`](https://github.com/srvcs/platform) for the shared standard.

> Note: the `cargoHash` in `flake.nix` is inherited from the template and must be
> refreshed with a `nix build` before the Nix gates pass.
