# Korelator

> *"An event unexamined is a threat undetected. The collective examines everything."*
> — Komrad Engineering Collective, 2026

Korelator is the correlation engine of the Komrad ecosystem. It consumes a stream of JSON events from Quickwit, evaluates each one against a set of declarative detection rules, and triggers alerts when a rule is satisfied. It consumes [Kompiler](https://github.com/komrad-company/Kompiler) for rule parsing and [Khronika](https://github.com/komrad-company/Khronika) for logging. Nothing else enters. Nothing else leaves without scrutiny.

```
JSON events (stdin for now — Quickwit ingestion pending)
    └──► Korelator (evaluation engine)
              ├── Kompiler   (rule parsing — Vec<Rule>)
              ├── Khronika   (logging — every decision is traced)
              └──► AlertSink (stderr JSON — pluggable trait)
```

The binary reads one JSON event per line from **stdin**, evaluates every rule
against every event, and emits an [`Alert`](#public-types) for each match
through the configured [`AlertSink`](#public-types). Malformed JSON lines are
skipped with a warning; an empty line is ignored.

Direct Quickwit ingestion is not yet wired — the stdin loop is the temporary
event source until that lands.

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
| `PreparedRule` | A parsed [`kompiler::Rule`] with its [`EvaluationContext`] built once at load time. Built via `From<Rule>`. Call `fires_on(&event)` to check the rule, `to_alert(event)` to materialise an `Alert`. |
| `Alert` | One detection record: `rule_id`, `title`, `level`, `event`, `timestamp_unix`. Serialises to JSON. |
| `AlertSink` | Trait — `fn emit(&self, &Alert)`. `Send + Sync`, ready to share across threads. |
| `StderrJsonSink` | Default sink — writes one JSON alert per line on stderr. |

### `Evaluate` trait

The evaluation contract for sub-rule fragments. Any structure that can be matched against a JSON event implements it.

```rust
pub trait Evaluate {
    fn evaluate(&self, event: &Value, ctx: &EvaluationContext) -> bool;
}
```

`EvaluationContext` carries shared named filters via `Arc<HashMap<String, Filters>>` — safe to share across threads. Implemented for `FieldFilter` and `Condition`. Returns `true` on the first matching value — implicit OR over the `values` list.

### Matcher support

| Matcher | Status |
|---|---|
| `Single` | ✅ Implemented — fires on every event satisfying the condition |
| `Threshold` | ⏳ Not yet implemented — rules using it are loaded but never fire; a warning is logged on each event |

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
