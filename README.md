# Korelator

![CI](https://github.com/komrad-company/Korelator/actions/workflows/ci.yml/badge.svg) ![Release](https://img.shields.io/github/v/release/komrad-company/Korelator) ![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue)

> *"An event unexamined is a threat undetected. The collective examines everything."*
> — Komrad Engineering Collective, 2026

Korelator is the correlation engine of the Komrad ecosystem. It consumes a stream of JSON events from Quickwit, evaluates each one against a set of declarative detection rules, and triggers alerts when a rule is satisfied. It consumes [Kompiler](https://github.com/komrad-company/Kompiler) for rule parsing and [Khronika](https://github.com/komrad-company/Khronika) for logging. Nothing else enters. Nothing else leaves without scrutiny.

```
Quickwit (JSON events)
    └──► Korelator (evaluation engine)
              ├── Kompiler   (rule parsing — Vec<Rule>)
              ├── Khronika   (logging — every decision is traced)
              └──► alert sink  [pending]
```

---

## Configuration

Korelator reads its configuration from a JSON file. The path is resolved from the `CONFIGURATION_PATH` environment variable, falling back to `configuration.json` in the working directory.

```json
{
    "quickwit_url": "http://quickwit.internal:7280",
    "rules_path": "/etc/korelator/rules",
    "log": {
        "level": "error",
        "file": "output/korelator.log"
    }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `quickwit_url` | `string` | ✅ | Quickwit instance URL — event source |
| `rules_path` | `string` | ✅ | Path to rule files, passed directly to `kompiler::parse_rules` |
| `log` | `TelemetryConfiguration` | ✅ | Khronika logger configuration |

A missing or malformed configuration file is fatal. The binary exits with code `1`. No partial start is tolerated.

---

## API

The collective exposes one function.

```rust
use korelator::load_configuration;

let config = load_configuration()?;
```

### Public types

| Type | Role |
|---|---|
| `Configuration` | Deserialised runtime configuration |

### `Evaluate` trait

The evaluation contract. Any structure that can be matched against a JSON event implements it.

```rust
pub trait Evaluate {
    fn evaluate(&self, event: &Value, ctx: &EvaluationContext) -> bool;
}
```

`EvaluationContext` carries shared named filters via `Arc<HashMap<String, Filters>>` — safe to share across threads. Currently implemented for `FieldFilter`. Returns `true` on the first matching value — implicit OR over the `values` list.

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Normal exit |
| `1` | Fatal error loading configuration |
| `2` | Fatal error parsing rules |

---

## Dependencies

Each dependency was evaluated by the collective before admission. None were added lightly.

| Crate | Source | Purpose |
|---|---|---|
| `kompiler` | komrad-company, git tag | Rule parsing and typed representation |
| `khronika` | komrad-company, git tag | Structured logging and telemetry |
| `serde` + `serde_json` | crates.io | Configuration deserialisation, event handling |

---

## License

AGPL-3.0-or-later — the source remains open, as all things should be.
