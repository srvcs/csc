# srvcs-csc

## Name

| Field | Value |
| --- | --- |
| Service | `srvcs-csc` |
| Slug | `csc` |
| Repository | `srvcs/csc` |
| Package | `srvcs-csc` |
| Kind | `orchestrator` |

## Function

trigonometry: cosecant

## Dependencies

| Dependency | Repository |
| --- | --- |
| `srvcs-sin` | [srvcs/sin](https://github.com/srvcs/sin) |
| `srvcs-floatdivide` | [srvcs/floatdivide](https://github.com/srvcs/floatdivide) |

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Service identity |
| `POST` | `/` | Evaluate the service function |
| `GET` | `/healthz` | Liveness probe |
| `GET` | `/readyz` | Readiness probe |
| `GET` | `/metrics` | Prometheus metrics |
| `GET` | `/openapi.json` | OpenAPI document |

## Inputs

| Name | Type | Required |
| --- | --- | --- |
| `value` | `json` | yes |

## Outputs

| Name | Type |
| --- | --- |
| `value` | `json` |
| `result` | `number` |

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `SRVCS_BIND_ADDR` | `0.0.0.0:8080` | Bind address |
| `SRVCS_ENV` | `development` | Environment label for logs |
| `RUST_LOG` | `info,tower_http=info` | Tracing filter |
| `SRVCS_FLOATDIVIDE_URL` | `http://127.0.0.1:8091` | Base URL for srvcs-floatdivide |
| `SRVCS_SIN_URL` | `http://127.0.0.1:8090` | Base URL for srvcs-sin |

## Error Behavior

- `422` means the request could not be evaluated for the documented input shape.
- `503` means a required dependency was unavailable or returned an unexpected response.
- Dependency validation errors are forwarded when this service delegates validation.

## Local Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

See the [srvcs service standard](https://github.com/srvcs/platform/blob/main/STANDARD.md) for the full operational contract.

## Metadata

Machine-readable service metadata lives in `srvcs.yaml`. Keep it aligned with this README when the service contract changes.
